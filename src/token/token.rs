use anyhow::Result;
use clap::Subcommand;
use pimalaya_tui::terminal::cli::printer::Printer;

use crate::account::Account;

use super::{inspect::InspectToken, refresh::RefreshToken, show::ShowToken};

#[derive(Subcommand, Debug)]
pub enum Token {
    Show(ShowToken),
    Inspect(InspectToken),
    Refresh(RefreshToken),
}

impl Token {
    pub fn execute(self, printer: &mut impl Printer, account: Account) -> Result<()> {
        match self {
            Self::Show(cmd) => cmd.execute(printer, account),
            Self::Inspect(cmd) => cmd.execute(printer, account),
            Self::Refresh(cmd) => cmd.execute(printer, account),
        }
    }
}
