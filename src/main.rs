use clap::Parser;
use ortie::cli::Cli;
use pimalaya_toolbox::terminal::{error::ErrorReport, log::Logger, printer::StdoutPrinter};

fn main() {
    let cli = Cli::parse();

    Logger::init(&cli.log);

    let mut printer = StdoutPrinter::new(&cli.json);
    let config_paths = cli.config.paths.as_ref();
    let account_name = cli.account.name.as_deref();

    let result = cli
        .command
        .execute(&mut printer, config_paths, account_name);

    ErrorReport::eval(&mut printer, result)
}
