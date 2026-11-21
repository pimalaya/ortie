//! # JSON Schema registry
//!
//! Maps a CLI-invocation key, the command path joined with hyphens and
//! prefixed `ortie-`, to the JSON Schema of that command's `--json`
//! payload. [`JsonSchemaCommand`] writes one file per entry, named after
//! its key the way the man pages are.
//!
//! Every command printing a value carries a key. The REPL carries none of
//! its own: it dispatches the commands of the binary and prints their
//! payloads unchanged. The one key naming no command is `ortie-error`,
//! the failure object any command prints instead of its payload. A
//! command printing events before its payload (`account check`) carries
//! their schema too, written as `<key>.events`.
//!
//! Every registered type is Ortie's own, so the machine contract stays a
//! decision of this repository rather than of the libraries below it.
//! The family's shared shape is the exception: the failure object, its
//! `code` typed by Ortie's vocabulary ([`crate::error`]) and
//! pimalaya-cli's `usage-invalid`.
//!
//! [`JsonSchemaCommand`]: pimalaya_cli::clap::commands::json_schema::JsonSchemaCommand

use pimalaya_cli::{clap::commands::json_schema::JsonSchemaRegistry, error::ErrorReport};

use crate::{
    account::{
        check::{AccountCheckEvent, AccountCheckOutput},
        list::AccountListOutput,
        show::AccountShowOutput,
    },
    auth::get::AuthGetOutput,
    error::ErrorCode,
    token::{
        inspect::TokenInspectOutput, issued::TokenIssuedOutput, revoke::TokenRevokeOutput,
        show::TokenShowOutput,
    },
};

/// Builds the command-to-schema registry consumed by `json-schema`.
///
/// Each value describes the type the command hands to the printer, as
/// printed, so the description cannot drift from the emission, and
/// `ortie-error` the failure object every command prints instead.
pub fn registry() -> JsonSchemaRegistry {
    let mut registry = JsonSchemaRegistry::new("ortie", env!("CARGO_PKG_VERSION"));

    registry.insert::<AccountListOutput>("ortie-account-list");
    registry.insert::<AccountShowOutput>("ortie-account-show");
    registry.insert::<AccountCheckOutput>("ortie-account-check");
    registry.insert_events::<AccountCheckEvent>("ortie-account-check");
    registry.insert::<AuthGetOutput<'static>>("ortie-auth-get");
    registry.insert::<TokenIssuedOutput>("ortie-auth-listen");
    registry.insert::<TokenIssuedOutput>("ortie-auth-resume");
    registry.insert::<TokenShowOutput<'static>>("ortie-token-show");
    registry.insert::<TokenInspectOutput>("ortie-token-inspect");
    registry.insert::<TokenIssuedOutput>("ortie-token-refresh");
    registry.insert::<TokenRevokeOutput>("ortie-token-revoke");
    registry.insert::<ErrorReport<ErrorCode>>("ortie-error");

    registry
}

#[cfg(test)]
mod tests {
    use clap::CommandFactory;
    use serde_json::json;

    use crate::{cli::Cli, json_schema::registry};

    /// The commands printing nothing a schema could describe: a bare `ortie`
    /// shows the help, and the REPL prints the payloads of the leaves it
    /// dispatches.
    const SILENT: &[&str] = &["ortie", "ortie-repl", "ortie-mcp"];

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

    /// The failure object's `code` is typed by Ortie's vocabulary and
    /// pimalaya-cli's `usage-invalid`.
    #[test]
    fn the_error_schema_names_the_codes() {
        let schemas = registry().schemas;
        let schema = schemas["ortie-error"].to_string();

        for code in [
            "grant-invalid",
            "permission-denied",
            "account-default-several",
            "usage-invalid",
        ] {
            assert!(schema.contains(&format!("\"{code}\"")), "{code}: {schema}");
        }
    }

    /// The account schemas require every key the commands always print,
    /// and none of the optional properties `account show` leaves out.
    #[test]
    fn the_account_schemas_require_what_is_always_printed() {
        let schemas = registry().schemas;
        let required = |schema: &serde_json::Value| schema["required"].clone();

        let list = &schemas["ortie-account-list"];
        let row = &list["$defs"]["AccountListRow"];
        assert_eq!(required(row), json!(["name", "default", "grant"]), "{list}");

        let check = &schemas["ortie-account-check"];
        assert_eq!(required(check), json!(["account", "checks"]), "{check}");
        let row = &check["$defs"]["AccountCheckRow"];
        let keys = json!(["check", "ok", "code", "error", "tls"]);
        assert_eq!(required(row), keys, "{check}");

        let show = &schemas["ortie-account-show"];
        assert_eq!(required(show), json!(["account"]), "{show}");
        let account = &show["$defs"]["AccountShow"];
        let keys = json!([
            "name",
            "default",
            "clientId",
            "grant",
            "endpoints",
            "alpn",
            "scopes",
            "resources",
            "pkce",
            "extras",
            "autoRefresh",
            "storage",
        ]);
        assert_eq!(required(account), keys, "{show}");
        let redacted = &show["$defs"]["AccountShowRedacted"];
        assert_eq!(required(redacted), json!(["raw", "redacted"]), "{show}");
    }

    /// `account check` describes its events apart, one per finished row,
    /// each carrying the account and the keys of the row.
    #[test]
    fn the_account_check_events_are_described() {
        let events = registry().events;
        let schema = &events["ortie-account-check"];
        let text = schema.to_string();

        assert!(text.contains("\"check-finished\""), "{schema}");

        for key in ["event", "account", "check", "ok", "code", "error", "tls"] {
            assert!(text.contains(&format!("\"{key}\"")), "{key}: {schema}");
        }
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
