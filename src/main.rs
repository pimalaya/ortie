use clap::Parser;
use ortie::cli::Cli;
use pimalaya_cli::{error::ErrorReport, log::Logger, printer::StdoutPrinter};

fn main() {
    let cli = Cli::parse();

    Logger::try_init(&cli.log).expect("init logger");
    let mut printer = StdoutPrinter::new(&cli.json);
    let config_paths = cli.config.paths.as_ref();
    let account_name = cli.account.name.as_deref();

    let result = cli.cmd.execute(&mut printer, config_paths, account_name);

    ErrorReport::eval(&mut printer, result)
}
