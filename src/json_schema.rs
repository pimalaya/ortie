//! # JSON Schema registry
//!
//! Maps a CLI-invocation key, the command path joined with hyphens and
//! prefixed `ortie-`, to the JSON Schema of that command's `--json`
//! payload. [`JsonSchemaCommand`] writes one file per entry, named after
//! its key the way the man pages are.
//!
//! Every command printing a value carries a key. The REPL carries none of
//! its own: it dispatches the same `token` and `auth` leaves and prints
//! their payloads unchanged. The one key naming no command is
//! `ortie-error`, the failure object any command prints instead of its
//! payload.
//!
//! Every registered type is Ortie's own, so the machine contract stays a
//! decision of this repository rather than of the libraries below it.
//! The failure object is the exception, being the family's shared one.
//!
//! [`JsonSchemaCommand`]: pimalaya_cli::clap::commands::JsonSchemaCommand

use pimalaya_cli::{clap::commands::JsonSchemaRegistry, error::ErrorReport};

#[cfg(feature = "wizard")]
use crate::wizard::ConfigureOutput;
use crate::{
    auth::get::AuthGetOutput,
    token::{inspect::TokenInspectOutput, issued::TokenIssuedOutput, show::TokenShowOutput},
};

/// Builds the command-to-schema registry consumed by `json-schema`.
///
/// Each value describes the type the command hands to the printer, as
/// printed, so the description cannot drift from the emission, and
/// `ortie-error` the failure object every command prints instead.
pub fn registry() -> JsonSchemaRegistry {
    let mut registry = JsonSchemaRegistry::new("ortie", env!("CARGO_PKG_VERSION"));

    #[cfg(feature = "wizard")]
    registry.insert::<ConfigureOutput>("ortie-configure");
    registry.insert::<AuthGetOutput<'static>>("ortie-auth-get");
    registry.insert::<TokenIssuedOutput>("ortie-auth-resume");
    registry.insert::<TokenShowOutput<'static>>("ortie-token-show");
    registry.insert::<TokenInspectOutput>("ortie-token-inspect");
    registry.insert::<TokenIssuedOutput>("ortie-token-refresh");
    registry.insert::<ErrorReport>("ortie-error");

    registry
}

#[cfg(test)]
mod tests {
    use clap::CommandFactory;

    use crate::cli::Cli;

    use super::*;

    /// The commands printing nothing a schema could describe: a bare `ortie`
    /// shows the help, or runs the wizard `configure` describes, and the REPL
    /// prints the payloads of the leaves it dispatches.
    const SILENT: &[&str] = &["ortie", "ortie-repl"];

    /// Every runnable command is described, and every key names one: a
    /// renamed subcommand cannot leave a schema nobody can ask for, nor a
    /// new one print undescribed data.
    #[test]
    fn the_registry_matches_the_command_tree() {
        assert_eq!(
            registry().check(&Cli::command(), SILENT),
            Vec::<String>::new()
        );
    }

    #[test]
    fn every_payload_is_an_object() {
        assert_eq!(registry().non_object_roots(), Vec::<String>::new());
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
