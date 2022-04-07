use std::{
    env,
    io::{self, Stdout, Write as _},
};

use log::{Level, LevelFilter, Log, Metadata, Record};
use owo_colors::OwoColorize as _;

use crate::prelude::*;

pub fn init() -> Result<()> {
    let bypass = env::var("VARELA_LOG_ALL").is_ok();
    let level = if bypass {
        LevelFilter::Trace
    } else {
        LevelFilter::Info
    };

    log::set_boxed_logger(Box::new(Logger {
        bypass,
        out: io::stdout(),
        level,
    }))?;
    log::set_max_level(level);

    Ok(())
}

pub struct Logger {
    bypass: bool,
    out: Stdout,
    level: LevelFilter,
}

impl Logger {
    fn print(&self, record: &Record<'_>) -> Result<()> {
        let mut out = self.out.lock();

        // TODO: use sometime like humantime (might have to fork) as chrono is kinda heavy
        // write!(
        //     &mut out,
        //     "{} ",
        //     Utc::now().format("%b %d %T").bright_black()
        // )?;

        #[allow(clippy::write_literal)]
        match record.level() {
            Level::Error => write!(&mut out, "{: <5} ", "ERROR".bright_red())?,
            Level::Warn => write!(&mut out, "{: <5} ", "WARN".bright_yellow())?,
            Level::Info => write!(&mut out, "{: <5} ", "INFO".bright_blue())?,
            Level::Debug => write!(&mut out, "{: <5} ", "DEBUG".green())?,
            Level::Trace => write!(&mut out, "{: <5} ", "TRACE")?,
        };

        write!(
            &mut out,
            "{: <21} ",
            record.target().trim_start_matches("varela_").cyan()
        )?;

        writeln!(&mut out, "{}", record.args())?;

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
