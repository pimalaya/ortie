use anyhow::Result;
use clap::Subcommand;
use pimalaya_toolbox::terminal::printer::Printer;

use crate::account::Account;

use super::{get::GetAuthorization, resume::ResumeAuthorization};

#[derive(Subcommand, Debug)]
pub enum Auth {
    Get(GetAuthorization),
    Resume(ResumeAuthorization),
}

impl Auth {
    pub fn execute(self, printer: &mut impl Printer, account: Account) -> Result<()> {
        match self {
            Self::Get(cmd) => cmd.execute(printer, account),
            Self::Resume(cmd) => cmd.execute(printer, account),
        }
    }
}
