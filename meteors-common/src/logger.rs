use {
    crate::prelude::*,
    chrono::Utc,
    log::{Level, LevelFilter, Log, Metadata, Record},
    owo_colors::OwoColorize as _,
    std::io::{self, Stdout, Write as _},
};

pub fn init() -> Result<()> {
    log::set_boxed_logger(Box::new(Logger {
        pid: std::process::id(),
        out: io::stdout(),
        level: LevelFilter::Trace,
    }))?;
    log::set_max_level(LevelFilter::Trace);

    Ok(())
}

pub struct Logger {
    pid: u32,
    out: Stdout,
    level: LevelFilter,
}

impl Logger {
    fn print(&self, record: &Record<'_>) -> Result<()> {
        if record.target().starts_with("tiny_http") {
            return Ok(());
        }

        let mut out = self.out.lock();

        write!(
            &mut out,
            "{} ",
            Utc::now().format("%d %b %Y %T%.3f").bright_black()
        )?;

        write!(
            &mut out,
            "{}[{}] ",
            "meteors".green(),
            self.pid.bright_purple(),
        )?;

        write!(
            &mut out,
            "{: <21} ",
            record.target().trim_start_matches("meteors_").bright_red()
        )?;

        #[allow(clippy::write_literal)]
        match record.level() {
            Level::Error => write!(&mut out, "[{: <5}] ", "ERROR".bright_red())?,
            Level::Warn => write!(&mut out, "[{: <5}] ", "WARN".bright_yellow())?,
            Level::Info => write!(&mut out, "[{: <5}] ", "INFO".bright_blue())?,
            Level::Debug => write!(&mut out, "[{: <5}] ", "DEBUG".green())?,
            Level::Trace => write!(&mut out, "[{: <5}] ", "TRACE")?,
        };

        writeln!(&mut out, "{}", record.args())?;

        Ok(())
    }
}

impl Log for Logger {
    fn enabled(&self, metadata: &Metadata<'_>) -> bool {
        metadata.level() <= self.level
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
