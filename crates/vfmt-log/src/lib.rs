#![feature(const_trait_impl)]

use std::{
    cmp,
    sync::atomic::{AtomicUsize, Ordering},
};

static mut LOGGER: &dyn Log = &NopLogger;
static STATE: AtomicUsize = AtomicUsize::new(0);

const UNINITIALIZED: usize = 0;
const INITIALIZING: usize = 1;
const INITIALIZED: usize = 2;

static MAX_LOG_LEVEL_FILTER: AtomicUsize = AtomicUsize::new(0);

static SET_LOGGER_ERROR: &str =
    "attempted to set a logger after the logging system was already initialized";

/// The statically resolved maximum log level.
///
/// See the crate level documentation for information on how to configure this.
///
/// This value is checked by the log macros, but not by the `Log`ger returned by
/// the [`logger`] function. Code that manually calls functions on that value
/// should compare the level against this value.
///
/// [`logger`]: fn.logger.html
pub const STATIC_MAX_LEVEL: LevelFilter = LevelFilter::Trace;

/// Sets the global maximum log level.
///
/// Generally, this should only be called by the active logging implementation.
///
/// Note that `Trace` is the maximum level, because it provides the maximum amount of detail in the emitted logs.
#[inline]
pub fn set_max_level(level: LevelFilter) {
    MAX_LOG_LEVEL_FILTER.store(level as usize, Ordering::Relaxed)
}

/// Returns the current maximum log level.
///
/// The [`log!`], [`error!`], [`warn!`], [`info!`], [`debug!`], and [`trace!`] macros check
/// this value and discard any message logged at a higher level. The maximum
/// log level is set by the [`set_max_level`] function.
///
/// [`log!`]: macro.log.html
/// [`error!`]: macro.error.html
/// [`warn!`]: macro.warn.html
/// [`info!`]: macro.info.html
/// [`debug!`]: macro.debug.html
/// [`trace!`]: macro.trace.html
/// [`set_max_level`]: fn.set_max_level.html
#[inline(always)]
pub fn max_level() -> LevelFilter {
    // Since `LevelFilter` is `repr(usize)`,
    // this transmute is sound if and only if `MAX_LOG_LEVEL_FILTER`
    // is set to a usize that is a valid discriminant for `LevelFilter`.
    // Since `MAX_LOG_LEVEL_FILTER` is private, the only time it's set
    // is by `set_max_level` above, i.e. by casting a `LevelFilter` to `usize`.
    // So any usize stored in `MAX_LOG_LEVEL_FILTER` is a valid discriminant.
    unsafe { std::mem::transmute(MAX_LOG_LEVEL_FILTER.load(Ordering::Relaxed)) }
}

pub fn set_boxed_logger(logger: Box<dyn Log>) -> Result<(), SetLoggerError> {
    set_logger_inner(|| Box::leak(logger))
}

fn set_logger_inner<F>(make_logger: F) -> Result<(), SetLoggerError>
where
    F: FnOnce() -> &'static dyn Log,
{
    let old_state = match STATE.compare_exchange(
        UNINITIALIZED,
        INITIALIZING,
        Ordering::SeqCst,
        Ordering::SeqCst,
    ) {
        Ok(s) | Err(s) => s,
    };
    match old_state {
        UNINITIALIZED => {
            unsafe {
                LOGGER = make_logger();
            }
            STATE.store(INITIALIZED, Ordering::SeqCst);
            Ok(())
        }
        INITIALIZING => {
            while STATE.load(Ordering::SeqCst) == INITIALIZING {
                std::hint::spin_loop();
            }
            Err(SetLoggerError(()))
        }
        _ => Err(SetLoggerError(())),
    }
}

/// Returns a reference to the logger.
///
/// If a logger has not been set, a no-op implementation is returned.
pub fn logger() -> &'static dyn Log {
    if STATE.load(Ordering::SeqCst) != INITIALIZED {
        static NOP: NopLogger = NopLogger;

        &NOP
    } else {
        unsafe { LOGGER }
    }
}

/// A trait encapsulating the operations required of a logger.
pub trait Log: Sync + Send {
    /// Determines if a log message with the specified metadata would be
    /// logged.
    ///
    /// This is used by the `log_enabled!` macro to allow callers to avoid
    /// expensive computation of log message arguments if the message would be
    /// discarded anyway.
    fn enabled(&self, metadata: &Metadata) -> bool;

    /// Logs the `Record`.
    ///
    /// Note that `enabled` is *not* necessarily called before this method.
    /// Implementations of `log` should perform all necessary filtering
    /// internally.
    fn log(&self, record: &Record);

    /// Flushes any buffered records.
    fn flush(&self);
}

#[derive(Copy, Eq)]
pub enum Level {
    Error = 1,
    Warn,
    Info,
    Debug,
    Trace,
}

impl Clone for Level {
    #[inline]
    fn clone(&self) -> Level {
        *self
    }
}

impl PartialEq for Level {
    #[inline]
    fn eq(&self, other: &Level) -> bool {
        *self as usize == *other as usize
    }
}

impl PartialEq<LevelFilter> for Level {
    #[inline]
    fn eq(&self, other: &LevelFilter) -> bool {
        *self as usize == *other as usize
    }
}

#[repr(usize)]
#[derive(Copy, Eq)]
pub enum LevelFilter {
    Off,
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl Clone for LevelFilter {
    #[inline]
    fn clone(&self) -> LevelFilter {
        *self
    }
}

impl PartialEq for LevelFilter {
    #[inline]
    fn eq(&self, other: &LevelFilter) -> bool {
        *self as usize == *other as usize
    }
}

impl PartialEq<Level> for LevelFilter {
    #[inline]
    fn eq(&self, other: &Level) -> bool {
        other.eq(self)
    }
}

impl PartialOrd<LevelFilter> for Level {
    #[inline]
    fn partial_cmp(&self, other: &LevelFilter) -> Option<cmp::Ordering> {
        Some((*self as usize).cmp(&(*other as usize)))
    }

    #[inline]
    fn lt(&self, other: &LevelFilter) -> bool {
        (*self as usize) < *other as usize
    }

    #[inline]
    fn le(&self, other: &LevelFilter) -> bool {
        *self as usize <= *other as usize
    }

    #[inline]
    fn gt(&self, other: &LevelFilter) -> bool {
        *self as usize > *other as usize
    }

    #[inline]
    fn ge(&self, other: &LevelFilter) -> bool {
        *self as usize >= *other as usize
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct Metadata<'a> {
    level: Level,
    target: &'a str,
}

impl<'a> Metadata<'a> {
    /// Returns a new builder.
    #[inline]
    pub fn builder() -> MetadataBuilder<'a> {
        MetadataBuilder::new()
    }

    /// The verbosity level of the message.
    #[inline]
    pub fn level(&self) -> Level {
        self.level
    }

    /// The name of the target of the directive.
    #[inline]
    pub fn target(&self) -> &'a str {
        self.target
    }
}

/// Builder for [`Metadata`](struct.Metadata.html).
///
/// Typically should only be used by log library creators or for testing and "shim loggers".
/// The `MetadataBuilder` can set the different parameters of a `Metadata` object, and returns
/// the created object when `build` is called.
///
/// # Example
///
/// ```edition2018
/// let target = "myApp";
/// use log::{Level, MetadataBuilder};
/// let metadata = MetadataBuilder::new()
///                     .level(Level::Debug)
///                     .target(target)
///                     .build();
/// ```
#[derive(Eq, PartialEq)]
pub struct MetadataBuilder<'a> {
    metadata: Metadata<'a>,
}

impl<'a> MetadataBuilder<'a> {
    /// Construct a new `MetadataBuilder`.
    ///
    /// The default options are:
    ///
    /// - `level`: `Level::Info`
    /// - `target`: `""`
    #[inline]
    pub fn new() -> MetadataBuilder<'a> {
        Self::default()
    }

    /// Setter for [`level`](struct.Metadata.html#method.level).
    #[inline]
    pub fn level(&mut self, arg: Level) -> &mut MetadataBuilder<'a> {
        self.metadata.level = arg;
        self
    }

    /// Setter for [`target`](struct.Metadata.html#method.target).
    #[inline]
    pub fn target(&mut self, target: &'a str) -> &mut MetadataBuilder<'a> {
        self.metadata.target = target;
        self
    }

    /// Returns a `Metadata` object.
    #[inline]
    pub fn build(&self) -> Metadata<'a> {
        self.metadata.clone()
    }
}

impl Default for MetadataBuilder<'_> {
    fn default() -> Self {
        Self {
            metadata: Metadata {
                level: Level::Info,
                target: "",
            },
        }
    }
}

#[derive(Clone)]
pub struct Record<'a> {
    metadata: Metadata<'a>,
    module_path: &'static str,
    file: &'static str,
    line: u32,
    message: String,
}

impl<'a> Record<'a> {
    /// Returns a new builder.
    #[inline]
    pub fn builder() -> RecordBuilder<'a> {
        RecordBuilder::new()
    }

    /// The message body.
    #[inline]
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Metadata about the log directive.
    #[inline]
    pub fn metadata(&self) -> &Metadata<'a> {
        &self.metadata
    }

    /// The verbosity level of the message.
    #[inline]
    pub fn level(&self) -> Level {
        self.metadata.level()
    }

    /// The name of the target of the directive.
    #[inline]
    pub fn target(&self) -> &'a str {
        self.metadata.target()
    }
}

/// Builder for [`Record`](struct.Record.html).
///
/// Typically should only be used by log library creators or for testing and "shim loggers".
/// The `RecordBuilder` can set the different parameters of `Record` object, and returns
/// the created object when `build` is called.
///
/// # Examples
///
/// ```edition2018
/// use log::{Level, Record};
///
/// let record = Record::builder()
///                 .args(format_args!("Error!"))
///                 .level(Level::Error)
///                 .target("myApp")
///                 .file(Some("server.rs"))
///                 .line(Some(144))
///                 .module_path(Some("server"))
///                 .build();
/// ```
///
/// Alternatively, use [`MetadataBuilder`](struct.MetadataBuilder.html):
///
/// ```edition2018
/// use log::{Record, Level, MetadataBuilder};
///
/// let error_metadata = MetadataBuilder::new()
///                         .target("myApp")
///                         .level(Level::Error)
///                         .build();
///
/// let record = Record::builder()
///                 .metadata(error_metadata)
///                 .args(format_args!("Error!"))
///                 .line(Some(433))
///                 .file(Some("app.rs"))
///                 .module_path(Some("server"))
///                 .build();
/// ```
pub struct RecordBuilder<'a> {
    record: Record<'a>,
}

impl<'a> RecordBuilder<'a> {
    /// Construct new `RecordBuilder`.
    ///
    /// The default options are:
    ///
    /// - `args`: [`format_args!("")`]
    /// - `metadata`: [`Metadata::builder().build()`]
    /// - `module_path`: `None`
    /// - `file`: `None`
    /// - `line`: `None`
    ///
    /// [`format_args!("")`]: https://doc.rust-lang.org/std/macro.format_args.html
    /// [`Metadata::builder().build()`]: struct.MetadataBuilder.html#method.build
    #[inline]
    pub fn new() -> RecordBuilder<'a> {
        Self::default()
    }

    /// Set [`message`](struct.Record.html#method.message).
    #[inline]
    pub fn message(&mut self, message: String) -> &mut RecordBuilder<'a> {
        self.record.message = message;
        self
    }

    /// Set [`metadata`](struct.Record.html#method.metadata). Construct a `Metadata` object with [`MetadataBuilder`](struct.MetadataBuilder.html).
    #[inline]
    pub fn metadata(&mut self, metadata: Metadata<'a>) -> &mut RecordBuilder<'a> {
        self.record.metadata = metadata;
        self
    }

    /// Set [`Metadata::level`](struct.Metadata.html#method.level).
    #[inline]
    pub fn level(&mut self, level: Level) -> &mut RecordBuilder<'a> {
        self.record.metadata.level = level;
        self
    }

    /// Set [`Metadata::target`](struct.Metadata.html#method.target)
    #[inline]
    pub fn target(&mut self, target: &'a str) -> &mut RecordBuilder<'a> {
        self.record.metadata.target = target;
        self
    }

    /// Set [`module_path`](struct.Record.html#method.module_path)
    #[inline]
    pub fn module_path(&mut self, path: &'static str) -> &mut RecordBuilder<'a> {
        self.record.module_path = path;
        self
    }

    /// Set [`module_path`](struct.Record.html#method.module_path) to a `'static` string
    #[inline]
    pub fn module_path_static(&mut self, path: &'static str) -> &mut RecordBuilder<'a> {
        self.record.module_path = path;
        self
    }

    /// Set [`file`](struct.Record.html#method.file)
    #[inline]
    pub fn file(&mut self, file: &'static str) -> &mut RecordBuilder<'a> {
        self.record.file = file;
        self
    }

    /// Set [`file`](struct.Record.html#method.file) to a `'static` string.
    #[inline]
    pub fn file_static(&mut self, file: &'static str) -> &mut RecordBuilder<'a> {
        self.record.file = file;
        self
    }

    /// Set [`line`](struct.Record.html#method.line)
    #[inline]
    pub fn line(&mut self, line: u32) -> &mut RecordBuilder<'a> {
        self.record.line = line;
        self
    }

    /// Set [`key_values`](struct.Record.html#method.key_values)
    #[cfg(feature = "kv_unstable")]
    #[inline]
    pub fn key_values(&mut self, kvs: &'a dyn kv::Source) -> &mut RecordBuilder<'a> {
        self.record.key_values = KeyValues(kvs);
        self
    }

    /// Invoke the builder and return a `Record`
    #[inline]
    pub fn build(&self) -> Record<'a> {
        self.record.clone()
    }
}

impl Default for RecordBuilder<'_> {
    fn default() -> Self {
        Self {
            record: Record {
                metadata: Metadata::builder().build(),
                module_path: "",
                file: "",
                line: 0,
                message: String::new(),
            },
        }
    }
}

struct NopLogger;

impl Log for NopLogger {
    fn enabled(&self, _: &Metadata) -> bool {
        false
    }

    fn log(&self, _: &Record) {}

    fn flush(&self) {}
}

#[derive(Debug)]
pub struct SetLoggerError(());

impl std::fmt::Display for SetLoggerError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        fmt.write_str(SET_LOGGER_ERROR)
    }
}

/// The standard logging macro.
///
/// This macro will generically log with the specified `Level` and `format!`
/// based argument list.
///
/// # Examples
///
/// ```edition2018
/// use vfmt_log::{log, Level};
///
/// # fn main() {
/// let data = (42, "Forty-two");
/// let private_data = "private";
///
/// log!(Level::Error, "Received errors: {}, {}", data.0, data.1);
/// log!(target: "app_events", Level::Warn, "App warning: {}, {}, {}",
///     data.0, data.1, private_data);
/// # }
/// ```
#[macro_export(local_inner_macros)]
macro_rules! log {
    // log!(target: "my_target", Level::Info; key1 = 42, key2 = true; "a {} event", "log");
    (target: $target:expr, $lvl:expr, $($key:tt = $value:expr),+; $($arg:tt)+) => ({
        let lvl = $lvl;
        if lvl <= $crate::STATIC_MAX_LEVEL && lvl <= $crate::max_level() {
            $crate::__private_api_log(
                __log_format_args!($($arg)+),
                lvl,
                &($target, __log_module_path!(), __log_file!(), __log_line!()),
                $crate::__private_api::Option::Some(&[$((__log_key!($key), &$value)),+])
            );
        }
    });

    // log!(target: "my_target", Level::Info; "a {} event", "log");
    (target: $target:expr, $lvl:expr, $($arg:tt)+) => ({
        let lvl = $lvl;
        if lvl <= $crate::STATIC_MAX_LEVEL && lvl <= $crate::max_level() {
            $crate::__private_api_log(
                lvl,
                &($target, __log_module_path!(), __log_file!(), __log_line!()),
                __log_format_args!($($arg)+),
            );
        }
    });

    // log!(Level::Info, "a log event")
    ($lvl:expr, $($arg:tt)+) => (log!(target: __log_module_path!(), $lvl, $($arg)+));
}

/// Logs a message at the error level.
///
/// # Examples
///
/// ```edition2018
/// use vfmt_log::error;
///
/// # fn main() {
/// let (err_info, port) = ("No connection", 22);
///
/// error!("Error: {} on port {}", err_info, port);
/// error!(target: "app_events", "App Error: {}, Port: {}", err_info, 22);
/// # }
/// ```
#[macro_export(local_inner_macros)]
macro_rules! error {
    // error!(target: "my_target", key1 = 42, key2 = true; "a {} event", "log")
    // error!(target: "my_target", "a {} event", "log")
    (target: $target:expr, $($arg:tt)+) => (log!(target: $target, $crate::Level::Error, $($arg)+));

    // error!("a {} event", "log")
    ($($arg:tt)+) => (log!($crate::Level::Error, $($arg)+))
}

/// Logs a message at the warn level.
///
/// # Examples
///
/// ```edition2018
/// use vfmt_log::warn;
///
/// # fn main() {
/// let warn_description = "Invalid Input";
///
/// warn!("Warning! {}!", warn_description);
/// warn!(target: "input_events", "App received warning: {}", warn_description);
/// # }
/// ```
#[macro_export(local_inner_macros)]
macro_rules! warn {
    // warn!(target: "my_target", key1 = 42, key2 = true; "a {} event", "log")
    // warn!(target: "my_target", "a {} event", "log")
    (target: $target:expr, $($arg:tt)+) => (log!(target: $target, $crate::Level::Warn, $($arg)+));

    // warn!("a {} event", "log")
    ($($arg:tt)+) => (log!($crate::Level::Warn, $($arg)+))
}

/// Logs a message at the info level.
///
/// # Examples
///
/// ```edition2018
/// use vfmt_log::info;
///
/// # fn main() {
/// # struct Connection { port: u32, speed: f32 }
/// let conn_info = Connection { port: 40, speed: 3.20 };
///
/// info!("Connected to port {} at {} Mb/s", conn_info.port, conn_info.speed);
/// info!(target: "connection_events", "Successfull connection, port: {}, speed: {}",
///       conn_info.port, conn_info.speed);
/// # }
/// ```
#[macro_export(local_inner_macros)]
macro_rules! info {
    // info!(target: "my_target", key1 = 42, key2 = true; "a {} event", "log")
    // info!(target: "my_target", "a {} event", "log")
    (target: $target:expr, $($arg:tt)+) => (log!(target: $target, $crate::Level::Info, $($arg)+));

    // info!("a {} event", "log")
    ($($arg:tt)+) => (log!($crate::Level::Info, $($arg)+))
}

/// Logs a message at the debug level.
///
/// # Examples
///
/// ```edition2018
/// use vfmt_log::debug;
///
/// # fn main() {
/// # struct Position { x: f32, y: f32 }
/// let pos = Position { x: 3.234, y: -1.223 };
///
/// debug!("New position: x: {}, y: {}", pos.x, pos.y);
/// debug!(target: "app_events", "New position: x: {}, y: {}", pos.x, pos.y);
/// # }
/// ```
#[macro_export(local_inner_macros)]
macro_rules! debug {
    // debug!(target: "my_target", key1 = 42, key2 = true; "a {} event", "log")
    // debug!(target: "my_target", "a {} event", "log")
    (target: $target:expr, $($arg:tt)+) => (log!(target: $target, $crate::Level::Debug, $($arg)+));

    // debug!("a {} event", "log")
    ($($arg:tt)+) => (log!($crate::Level::Debug, $($arg)+))
}

/// Logs a message at the trace level.
///
/// # Examples
///
/// ```edition2018
/// use vfmt_log::trace;
///
/// # fn main() {
/// # struct Position { x: f32, y: f32 }
/// let pos = Position { x: 3.234, y: -1.223 };
///
/// trace!("Position is: x: {}, y: {}", pos.x, pos.y);
/// trace!(target: "app_events", "x is {} and y is {}",
///        if pos.x >= 0.0 { "positive" } else { "negative" },
///        if pos.y >= 0.0 { "positive" } else { "negative" });
/// # }
/// ```
#[macro_export(local_inner_macros)]
macro_rules! trace {
    // trace!(target: "my_target", key1 = 42, key2 = true; "a {} event", "log")
    // trace!(target: "my_target", "a {} event", "log")
    (target: $target:expr, $($arg:tt)+) => (log!(target: $target, $crate::Level::Trace, $($arg)+));

    // trace!("a {} event", "log")
    ($($arg:tt)+) => (log!($crate::Level::Trace, $($arg)+))
}

/// Determines if a message logged at the specified level in that module will
/// be logged.
///
/// This can be used to avoid expensive computation of log message arguments if
/// the message would be ignored anyway.
///
/// # Examples
///
/// ```edition2018
/// use vfmt_log::Level::Debug;
/// use vfmt_log::{debug, log_enabled};
///
/// # fn foo() {
/// if log_enabled!(Debug) {
///     let data = expensive_call();
///     debug!("expensive debug data: {} {}", data.x, data.y);
/// }
/// if log_enabled!(target: "Global", Debug) {
///    let data = expensive_call();
///    debug!(target: "Global", "expensive debug data: {} {}", data.x, data.y);
/// }
/// # }
/// # struct Data { x: u32, y: u32 }
/// # fn expensive_call() -> Data { Data { x: 0, y: 0 } }
/// # fn main() {}
/// ```
#[macro_export(local_inner_macros)]
macro_rules! log_enabled {
    (target: $target:expr, $lvl:expr) => {{
        let lvl = $lvl;
        lvl <= $crate::STATIC_MAX_LEVEL
            && lvl <= $crate::max_level()
            && $crate::__private_api_enabled(lvl, $target)
    }};
    ($lvl:expr) => {
        log_enabled!(target: __log_module_path!(), $lvl)
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __log_format_args {
    ($($args:tt)*) => {{
        use $crate::__private_api::vfmt::{self, uwrite};

        |fmt| {
            let _ = uwrite!(fmt, $( $args )*);
        }
    }};
}

#[doc(hidden)]
#[macro_export]
macro_rules! __log_module_path {
    () => {
        module_path!()
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __log_file {
    () => {
        file!()
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __log_line {
    () => {
        line!()
    };
}

// WARNING: this is not part of the crate's public API and is subject to change at any time
#[doc(hidden)]
pub mod __private_api {
    pub use std::option::Option;
    pub use vfmt;
}

// WARNING: this is not part of the crate's public API and is subject to change at any time
#[doc(hidden)]
pub fn __private_api_enabled(level: Level, target: &str) -> bool {
    logger().enabled(&Metadata::builder().level(level).target(target).build())
}

// WARNING: this is not part of the crate's public API and is subject to change at any time
#[doc(hidden)]
pub fn __private_api_log<F: FnOnce(&mut String)>(
    level: Level,
    info: &(&str, &'static str, &'static str, u32),
    formatter: F,
) {
    fn inner(
        level: Level,
        &(target, module_path, file, line): &(&str, &'static str, &'static str, u32),
        message: String,
    ) {
        logger().log(
            &Record::builder()
                .message(message)
                .level(level)
                .target(target)
                .module_path_static(module_path)
                .file_static(file)
                .line(line)
                .build(),
        );
    }

    let mut buf = String::new();

    formatter(&mut buf);

    inner(level, info, buf);
}
