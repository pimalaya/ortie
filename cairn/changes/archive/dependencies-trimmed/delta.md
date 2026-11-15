---
cairn: delta
change: dependencies-trimmed
---

## ADDED Requirements

### Requirement: Hosts are ASCII
Ortie SHALL build url's IDNA without a Unicode back end, `idna_adapter` being pinned to 1.0.0 in Cargo.toml. A host SHALL be written in ASCII, an internationalised one in its Punycode form (`xn--…`), and a URL with a non-ASCII host SHALL be refused when parsed.

#### Scenario: An internationalised token endpoint
- **GIVEN** `endpoints.token = "https://bücher.example/token"`
- **WHEN** the configuration is read
- **THEN** it is refused, and the same account is read with `endpoints.token = "https://xn--bcher-kva.example/token"`

### Requirement: Dependencies name their features
Every dependency SHALL be declared with `default-features = false` and the features Ortie uses, so a default turned on upstream never reaches the binary. clap SHALL be built without `color` and `wrap_help`, and pimalaya-cli without env_logger's `auto-color`: help, usage and logs carry no colour, nothing asks the terminal for its size, and help prints each paragraph on one line.

## MODIFIED Requirements

### Requirement: TLS-only feature set
The cargo features SHALL be the TLS providers (rustls-ring by default) and `vendored`. The non-TLS features (`oauth2`/`rfc6749`, `command`, `cli`, `client`, `notify`) are dropped.

### Requirement: Hooks
An account MAY define hooks fired on token issuance and refresh, each with success and error variants. A hook MAY run a command, with the token or error exposed as environment variables; a desktop notification is such a command (`notify-send`, `osascript`). Secrets travel as secret strings and are never logged.

### Requirement: v1 config compatibility
Every v1.x config file SHALL parse and run unchanged, a hook `notify` table aside, refused like any unknown key: `grant` defaults to authorization-code, `endpoints.authorization` merely becomes optional, and `pkce = true`/`false` are still accepted. The only behaviour an existing account can notice is PKCE-by-default.

### Requirement: Authorization code grant
On an authorization-code account, `auth get` SHALL build the authorization URL (with PKCE per the account's posture, a generated `state`, and any `extras`), print it for the user to open, and capture the redirect on a loopback listener, then exchange the code, write storage, and fire the on-issue hooks. Ortie SHALL NOT launch a browser.

### Requirement: Nothing bypasses the printer under JSON
Every write to stdout that is not the printer's SHALL be guarded by the JSON mode, so a `--json` run emits the payload and nothing else. The handoff instructions and the wait notices of `auth get` are human text on the human path only.

## REMOVED Requirements
