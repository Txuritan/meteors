use std::env;

use vfmt::{stdio::Stdout, uwrite, uwriteln};
use vfmt_log::{Level, LevelFilter, Log, Metadata, Record};

use crate::{colorize::Colorize as _, prelude::*};

pub fn init() -> Result<()> {
    let bypass = env::var("VARELA_LOG_ALL").is_ok();
    let level = if bypass {
        LevelFilter::Trace
    } else {
        LevelFilter::Info
    };

    vfmt_log::set_boxed_logger(Box::new(Logger { bypass, level }))
        .map_err(|_| anyhow!("unable to set logger"))?;
    vfmt_log::set_max_level(level);

    Ok(())
}

pub struct Logger {
    bypass: bool,
    level: LevelFilter,
}

impl Logger {
    fn print(&self, record: &Record<'_>) -> Result<()> {
        // let mut out = self.out.lock();

        // TODO: use sometime like humantime (might have to fork) as chrono is kinda heavy
        // write!(
        //     &mut out,
        //     "{} ",
        //     Utc::now().format("%b %d %T").bright_black()
        // )?;

        #[rustfmt::skip]
        #[allow(clippy::write_literal)]
        let _ = match record.level() {
            Level::Error => uwrite!(Stdout, "{: <5} ", "ERROR".bright_red()),
            Level::Warn =>  uwrite!(Stdout, "{: <5} ", "WARN".bright_yellow()),
            Level::Info =>  uwrite!(Stdout, "{: <5} ", "INFO".bright_blue()),
            Level::Debug => uwrite!(Stdout, "{: <5} ", "DEBUG".green()),
            Level::Trace => uwrite!(Stdout, "{: <5} ", "TRACE"),
        };

        let _ = uwrite!(
            Stdout,
            "{: <21} ",
            record.target().trim_start_matches("varela_").cyan()
        );

        let _ = uwriteln!(Stdout, "{}", record.message());

        Ok(())
    }
}

impl Log for Logger {
    fn enabled(&self, metadata: &Metadata<'_>) -> bool {
        self.bypass | (metadata.level() <= self.level)
    }

    fn log(&self, record: &Record<'_>) {
        if !self.enabled(record.metadata()) {
            return;
        }

        if let Err(err) = self.print(record) {
            println!("unable to print log: {}", err);
        }
    }

    fn flush(&self) {}
}
