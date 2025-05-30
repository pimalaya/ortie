use std::{
    backtrace::{Backtrace, BacktraceStatus},
    env, fmt, process,
};

use anyhow::Error;
use clap::Parser;
use log::{log_enabled, Level};
use ortie::cli::Cli;
use pimalaya_tui::terminal::cli::printer::{OutputFmt, Printer, StdoutPrinter};
use serde::{ser::SerializeStruct, Serialize, Serializer};

fn main() {
    let cli = Cli::parse();

    if cli.quiet {
        env::set_var("RUST_LOG", "off");
    } else if cli.debug {
        env::set_var("RUST_LOG", "debug");
    } else if cli.trace {
        env::set_var("RUST_LOG", "trace");
        env::set_var("RUST_BACKTRACE", "1");
    }

    env_logger::init();

    let mut printer = StdoutPrinter::new(if cli.json {
        OutputFmt::Json
    } else {
        OutputFmt::Plain
    });

    let config_paths = cli.config_paths.as_ref();
    let account_name = cli.account_name.as_deref();

    let exec = cli
        .command
        .execute(&mut printer, config_paths, account_name);

    if let Err(err) = exec {
        printer
            .out(ErrorReport::from(err))
            .expect("should write error report to stdout");

        process::exit(1);
    }
}

pub struct ErrorReport(Error);

impl ErrorReport {
    fn sources(&self) -> impl Iterator<Item = String> + '_ {
        self.0.chain().skip(1).map(ToString::to_string)
    }

    fn suggestions(&self) -> Vec<&str> {
        let mut suggestions = Vec::with_capacity(3);

        if !log_enabled!(Level::Debug) {
            suggestions.push("Run with --debug to enable debug logs");
        }

        let backtrace = matches!(self.0.backtrace().status(), BacktraceStatus::Disabled);
        if !log_enabled!(Level::Trace) && !backtrace {
            suggestions.push("Run with --trace to enable verbose logs with backtraces");
        }

        suggestions
    }

    fn backtrace(&self) -> Option<&Backtrace> {
        let backtrace = self.0.backtrace();

        if let BacktraceStatus::Captured = backtrace.status() {
            Some(backtrace)
        } else {
            None
        }
    }
}

impl From<Error> for ErrorReport {
    fn from(err: Error) -> Self {
        Self(err)
    }
}

impl fmt::Display for ErrorReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Error: {}", self.0)?;

        let mut header_printed = false;
        for err in self.sources() {
            if !header_printed {
                writeln!(f)?;
                writeln!(f)?;
                write!(f, "Caused by:")?;
                header_printed = true;
            }

            writeln!(f)?;
            write!(f, " - {}", err.trim())?;
        }

        if let Some(backtrace) = self.backtrace() {
            writeln!(f)?;
            writeln!(f)?;
            writeln!(f, "Backtrace:")?;
            write!(f, "{backtrace}")?;
        }

        let mut header_printed = false;
        for suggestion in self.suggestions() {
            if !header_printed {
                writeln!(f)?;
                writeln!(f)?;
                write!(f, "Suggestions:")?;
                header_printed = true;
            }

            writeln!(f)?;
            write!(f, " - {suggestion}")?;
        }

        Ok(())
    }
}

impl Serialize for ErrorReport {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let sources: Vec<_> = self.0.chain().skip(1).map(ToString::to_string).collect();
        let backtrace = self.backtrace().map(ToString::to_string);

        let mut s = serializer.serialize_struct("ErrorReport", 3)?;
        s.serialize_field("error", &self.0.to_string())?;
        s.serialize_field("sources", &sources)?;
        s.serialize_field("backtrace", &backtrace)?;
        s.end()
    }
}
