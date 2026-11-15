---
cairn: delta
change: tls-table
---

# Delta

Each requirement names its capability file in parentheses.

## MODIFIED Requirements

### Requirement: Storage commands
(config) An account SHALL define read and write storage as external shell commands, each keyed `command`, `cmd` its alias, as a shell line or a program and its arguments (pimalaya-config's `CommandConfig`). The read command prints the token response JSON on stdout; the write command receives it on stdin. Ortie never persists tokens itself.

### Requirement: v1 config compatibility
(config) Every v1.x config file SHALL parse and run unchanged, the `hooks` table and the `tls` string aside, refused by name ("The configuration refuses every key it does not read"): `grant` defaults to authorization-code, `endpoints.authorization` merely becomes optional, and `pkce = true`/`false` are still accepted. The behaviours an existing account can notice are PKCE by default and `token show` refreshing an expired token, unless it sets `auto-refresh = false`.

### Requirement: One TLS conversion, taking the ALPN list
(config) The `tls` table of an account SHALL hold `provider` (`rustls` or `native-tls`) and `rustls.crypto` (`ring` or `aws`), each the first the build carries when omitted, and `cert`, a PEM certificate to trust beside the platform store, its path shell-expanded. It SHALL be folded into the runtime TLS handle by a single conversion taking the ALPN list as its argument. No other code path SHALL build that handle from the configuration, so no call site can negotiate an ALPN, or none, by accident.

#### Scenario: Omitted TLS table
- GIVEN an account with no `tls` key
- WHEN a command connects to the token endpoint
- THEN the provider and the crypto are the first the binary was built with, and no extra certificate is trusted

### Requirement: One sample configuration, with a section per provider
(config) The repository SHALL ship config.sample.toml, and the release archives and the Nix package SHALL install it as share/doc/ortie/config.sample.toml, which install.sh copies under its prefix. It holds one `[accounts.example]` table with dotted keys and no nested table. It SHALL list the fields every provider shares and each of their values, the default active and the others commented as `#key = value`, a field whose default is to be left out (the `tls` keys) commented whole, every line ending in one tab-aligned sentence, and every command a failing `exit 127` placeholder. It SHALL end with one section per provider (Google, Microsoft, Fastmail), Microsoft having one for Graph and one for IMAP and SMTP since Entra issues a token for one resource, and a custom one for an application of one's own at any other provider, each a paragraph of commented code only, headed by its name, holding the client, the endpoints, the redirection, the resources and the scopes the provider needs, Pimalaya's application where one is released, an empty `client-id` and `client-secret.raw` in the Google one and example values in the custom one. The file SHALL hold a working account once one section is uncommented, its client filled in where left empty. The test suite SHALL parse config.sample.toml through the loader `-c` uses with each section uncommented whole, then on top of it each commented element inside it and each line of commented code outside the sections, uncommented in place of the active lines of its field, so a configuration change that breaks one fails the build.

#### Scenario: A field is renamed
- GIVEN config.sample.toml, whose commented code sets `client-secret.raw`
- WHEN the field is renamed in the configuration types and the sample is not updated
- THEN the test suite fails naming the sample

### Requirement: The configuration refuses every key it does not read
(config) The configuration SHALL refuse at load, naming it, every key it does not read, at every level: the document root, which holds `accounts` alone; an account; and every table nested in one (`endpoints`, `tls`, `tls.rustls`, `storage`, `storage.read`, `storage.write`, `client-secret`). `extras` alone takes any key, its keys being wire parameter names. The `hooks` table, which Ortie read until 3.0.0, SHALL be refused by name whatever it holds, and so SHALL the `tls` string Ortie read until then, each in pimalaya-config's family sentence, "The `<name>` key (or table) is removed, <why>: <what to write instead>, see MIGRATION.md": the `hooks` one naming `storage.write.command`, the `tls` one the keys its value becomes.

#### Scenario: A typo in a nested table
- GIVEN an account carrying `endpoints.tokn = "https://login.example.com/token"`
- WHEN any command loads the configuration
- THEN it fails naming the unknown field `tokn`

#### Scenario: A 2.x hook
- GIVEN an account carrying `hooks.on-issue.success.command = "notify-send Ortie"`
- WHEN any command loads the configuration
- THEN it fails with "The `hooks` table is removed, the storage write command running on every issue and refresh: chain a command after `storage.write.command`, see MIGRATION.md"

#### Scenario: A 2.x TLS string
- GIVEN an account carrying `tls = "rustls-ring"`
- WHEN any command loads the configuration
- THEN it fails with "The `tls` key is removed, the TLS options being a table as in every Pimalaya CLI: write `tls.provider = "rustls"` and `tls.rustls.crypto = "ring"`, see MIGRATION.md"
