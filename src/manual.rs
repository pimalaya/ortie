use std::{fs, path::PathBuf};

use anyhow::Result;
use clap::{CommandFactory, Parser};
use clap_mangen::Man;
use log::info;
use pimalaya_toolbox::terminal::{cli::path_parser, printer::Printer};

use crate::cli::Cli;

/// Generate manual pages to the given directory.
///
/// This command allows you to generate manual pages (following the
/// man page format) to the given directory. If the directory does not
/// exist, it will be created. Any existing man pages will be
/// overriden.
#[derive(Debug, Parser)]
pub struct GenerateManuals {
    /// Directory where man files should be generated in.
    #[arg(value_parser = path_parser)]
    pub dir: PathBuf,
}

impl GenerateManuals {
    pub fn execute(self, printer: &mut impl Printer) -> Result<()> {
        let dir = &self.dir;
        let cmd = Cli::command();
        let cmd_name = cmd.get_name().to_string();
        let subcmds = cmd.get_subcommands().cloned().collect::<Vec<_>>();
        let subcmds_len = subcmds.len() + 1;

        let mut buffer = Vec::new();
        Man::new(cmd).render(&mut buffer)?;

        fs::create_dir_all(&dir)?;
        info!("generate man page for command {cmd_name}");
        fs::write(dir.join(format!("{}.1", cmd_name)), buffer)?;

        for subcmd in subcmds {
            let subcmd_name = subcmd.get_name().to_string();

            let mut buffer = Vec::new();
            Man::new(subcmd).render(&mut buffer)?;

            info!("generate man page for subcommand {subcmd_name}");
            fs::write(dir.join(format!("{}-{}.1", cmd_name, subcmd_name)), buffer)?;
        }

        printer.out(format!(
            "{subcmds_len} man page(s) successfully generated in {}",
            dir.display()
        ))
    }
}
