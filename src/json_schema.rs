//! # JSON Schema registry
//!
//! Maps a CLI-invocation key, the command path joined with hyphens and
//! prefixed `ortie-`, to the JSON Schema of that command's `--json`
//! payload. [`JsonSchemaCommand`] writes one file per entry, named after
//! its key the way the man pages are.
//!
//! Only the commands emitting data carry a key. `token refresh` and
//! `auth resume` print a confirmation message, so they carry none, and
//! the REPL carries none of its own: it dispatches the same `token` and
//! `auth` leaves and prints their payloads unchanged. The one key naming
//! no command is `ortie-error`, the failure object any command prints
//! instead of its payload.
//!
//! Every registered type is Ortie's own, so the machine contract stays a
//! decision of this repository rather than of the libraries below it.
//! The failure object is the exception, being the family's shared one.
//!
//! [`JsonSchemaCommand`]: pimalaya_cli::clap::commands::JsonSchemaCommand

use std::collections::BTreeMap;

use pimalaya_cli::{clap::commands::JsonSchemaRegistry, error::ErrorReport};
use schemars::schema_for;

#[cfg(feature = "wizard")]
use crate::wizard::ConfigureOutput;
use crate::{
    auth::get::AuthGetOutput,
    token::{inspect::TokenInspectOutput, show::TokenShowOutput},
};

/// The registry key of the failure object, which names no command.
const ERROR_KEY: &str = "ortie-error";

/// Builds the command-to-schema registry consumed by `json-schema`.
///
/// Each value describes the type the command hands to the printer, so
/// the description cannot drift from the emission, and `ortie-error`
/// the failure object every command prints instead.
pub fn registry() -> JsonSchemaRegistry {
    let mut schemas = BTreeMap::new();

    macro_rules! insert {
        ($key:expr, $ty:ty) => {
            schemas.insert(
                $key.to_string(),
                serde_json::to_value(schema_for!($ty)).unwrap(),
            );
        };
    }

    #[cfg(feature = "wizard")]
    insert!("ortie-configure", ConfigureOutput);
    insert!("ortie-auth-get", AuthGetOutput<'static>);
    insert!("ortie-token-show", TokenShowOutput<'static>);
    insert!("ortie-token-inspect", TokenInspectOutput);
    insert!(ERROR_KEY, ErrorReport);

    JsonSchemaRegistry {
        bin: "ortie",
        version: env!("CARGO_PKG_VERSION"),
        schemas,
    }
}

#[cfg(test)]
mod tests {
    use clap::{Command, CommandFactory};

    use crate::cli::Cli;

    use super::*;

    /// Whether `path` walks down to a real subcommand of `cmd`.
    ///
    /// A command name may itself hold a hyphen, so each step tries the
    /// longest join first and backtracks.
    fn resolves(cmd: &Command, path: &[&str]) -> bool {
        if path.is_empty() {
            return true;
        }

        (1..=path.len()).rev().any(|take| {
            let name = path[..take].join("-");

            cmd.get_subcommands()
                .find(|sub| sub.get_name() == name)
                .is_some_and(|sub| resolves(sub, &path[take..]))
        })
    }

    #[test]
    fn every_key_names_a_real_command() {
        let cli = Cli::command();

        for cmd in registry().schemas.keys() {
            if cmd == ERROR_KEY {
                continue;
            }

            let path: Vec<&str> = cmd.split('-').collect();

            assert_eq!(path[0], cli.get_name(), "{cmd} is not an ortie invocation");
            assert!(resolves(&cli, &path[1..]), "{cmd} names no command");
        }
    }

    #[test]
    fn every_key_builds_an_object_schema() {
        let schemas = registry().schemas;

        assert_eq!(schemas.len(), if cfg!(feature = "wizard") { 5 } else { 4 });

        for (cmd, schema) in schemas {
            assert!(cmd.starts_with("ortie-"), "{cmd} is not a CLI invocation");
            assert!(schema.is_object(), "{cmd} schema is not an object");
        }
    }

    #[test]
    fn every_payload_is_an_object() {
        assert_eq!(registry().non_object_roots(), Vec::<&str>::new());
    }

    #[test]
    fn token_inspect_schema_describes_no_secret() {
        let schemas = registry().schemas;
        let schema = schemas["ortie-token-inspect"].to_string();

        assert!(!schema.contains("\"accessToken\""));
        assert!(!schema.contains("\"refreshToken\""));
        assert!(schema.contains("\"withRefreshToken\""));
    }
}
