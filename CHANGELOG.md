# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [2.2.0] - 2026-08-15

### Added

- Added the `configure` command (alias `wizard`), running the account wizard by name.

  A bare `ortie` and any command needing an account offer it when they find no configuration, behind a welcome naming the file they looked for. The offer is a hook rather than a gate: the command carries on afterwards either way. Nothing prompts when stdin is not a terminal or `--json` is set.

- Added the `ORTIE_CONFIG` environment variable, read like `-c`, both now accepting a `:`-delimited list.
- Added the shared Pimalaya help footer, pointing at the bug tracker and the sponsoring page.

### Changed

- Changed a bare `ortie` to show the help when it finds a configuration, instead of always running the wizard.

  With no configuration it offers to create one, and `--account` with no subcommand shows the help too.

- Dropped the wizard's save-path prompt, writing where `-c` or ORTIE_CONFIG pointed, or to config.toml under $XDG_CONFIG_HOME/ortie.

  The account is still printed before the save is offered, and an existing file is still appended to rather than overwritten, after confirmation. The generated account takes a name the configuration does not already hold, suffixed until free, and claims `default = true` only when no other account does.

- Named what is missing in the three account resolution failures: the configuration path read, the accounts it holds, and the two ways to pick a default.

  They used to read Config file not found and Account not found.

- Bumped io-pim-discovery to 0.6 and pimalaya-stream to 0.2.

## [2.1.0] - 2026-08-11

### Added

- Added the `repl` command: a persistent session bound to one account that reads the secret store once and answers `token` and `auth` commands over stdin/stdout.

  One-shot commands re-read the secret store on every run, so a keyring that confirms disclosure per process prompts again and again. The REPL resolves the token once, holds it in memory for the life of the process, and reuses it, collapsing those prompts into a single unlock (plus one per refresh, which must persist a rotated refresh token). It shows a prompt on a terminal and uses a plain line protocol when piped (one flushed result per command on stdout, errors on stderr), so an application can drive it without reimplementing the OAuth flow. `auth get` and `auth resume` run against the same in-memory account, so a token they issue is immediately served by a following `token show`.

- Added the device authorization grant (RFC 8628).

  `grant = "device"` with `endpoints.device-authorization`. Interactive `auth get` polls; non-interactive / `--json` hands off to `auth resume <DEVICE_CODE>`.

- Added the headless client credentials grants, so OAuth-ignorant tools (a sync daemon, a cron job) can exec `ortie token show --auto-refresh` and always read a valid bearer token.

  `grant = "client-credentials"` (RFC 6749 section 4.4) authenticates with the existing `client-secret`; `grant = "client-credentials-jwt"` (RFC 7523 section 2.2, the Microsoft certificate credentials flow) authenticates with a JWT assertion signed by the new `client-key` (PKCS#8 or PKCS#1 PEM path), carrying the `x5t` thumbprint of the new `client-certificate` (PEM or DER path). Both run `auth get` headlessly in one shot; `auth resume` has nothing to resume and says so.

  Neither grant issues a refresh token, so auto-refresh (`token show --auto-refresh` and `token refresh`) branches per grant: refresh-token exchange where one exists, silent re-acquisition (a re-run of the grant) for the client credentials kinds, including when the stored token is missing, so the very first run needs no prior `auth get`. Each JWT re-acquisition mints a fresh 10 minute assertion (unique `jti`, key and certificate re-read from disk); assertions are never stored.

  A provider `invalid_client` on the JWT kind now hints that the certificate credential may be expired and need renewal.

- Added a welcome banner to the wizard, on stderr, framing what ortie is and where every field is documented.
- Added a keyring entry prompt to the wizard's storage step, seeded with the account name and used verbatim.

  It matches the Himalaya wizard, where the entry used to be namespaced under `ortie/` on your behalf.

- Offered to save the wizard's account to a config file, defaulting to `$XDG_CONFIG_HOME/ortie/config.toml`.

  The account is printed first and the save comes after it, so the prompt decides one thing only and declining leaves you the printed fragment. An existing file is appended to, never rewritten: the fragment is one `[accounts.<name>]` table, so the accounts and comments already in it are untouched. Appending to a file that already holds something is confirmed first, naming the path. A redirected stdout (or `--json`) prints without prompting at all, so `ortie >> <config>` is unchanged.

- Resolved issuers into concrete grants: a typed issuer URL, and a discovered OAuth issuer, are read through their RFC 8414 metadata instead of being emitted as a bare issuer comment.

  Every grant the metadata advertises is offered, so a server publishing both an authorization endpoint and a device authorization endpoint yields both flows, as RFC 8414 and RFC 8628 allow, and a machine with no browser can pick the device grant where a redirect is also possible.

- Offered the authorization server's `scopes_supported` in the scope multi-select, for an application not bound to a registered set of scopes.

### Changed

- Bumped io-oauth to 0.2.1, for the RFC 7523 support behind its `jwt-bearer` feature, and io-pim-discovery to 0.5, for the time-bounded discovery the wizard now runs.
- Stamped a refreshed or issued token with the local clock instead of relying on the server `Date` header.

  A missing `expires_in` defaults to one hour, so auto-refresh stays reliable across a long-lived session.

- Emitted keyring read commands as an exec-style array instead of a shell string, so no shell reinterprets an entry name.

  Write commands stay shell lines, since some rely on shell features (`$(cat)` on macOS), as do commands typed by hand.

- Ordered the wizard's storage strategies by what is installed, the credential provider CLIs found on your `PATH` leading.

  The ones that are not found are still offered, and say so: each is one package install away, and the commands written for it are correct either way.

- Bounded the wizard's discovery with a 6 second deadline, so one unreachable endpoint cannot stall the prompt.
- Reduced discovered grants by authorization server rather than by endpoint URL, so one server described by two mechanisms is offered once instead of twice.

  Gmail no longer asks you to choose between Mozilla autoconfig's legacy `accounts.google.com/o/oauth2/auth` + `www.googleapis.com/oauth2/v3/token` pair and the current one: the most authoritative mechanism wins, and an entry is labelled by its flow and services instead of its token endpoint. A pick list left with a single grant is no longer prompted for.

- Asked for the application before the scopes, since a registration is what decides which scopes can be requested.

  A public application now offers exactly the scopes it is registered for (Thunderbird on Google: Gmail, CardDAV and CalDAV), so a scope its client id was never verified for can no longer reach the authorization request and fail at consent. Dynamic registration picks its scopes before it registers, since they travel in the registration request.

- Derived the wizard's account name from the input (the first label of the domain or issuer host) instead of prompting for it; rename the `[accounts.<name>]` key by hand.
- Moved the wizard's guidance from the printed fragment into the stderr banner, so stdout now carries bare TOML.
- Probed the authorization server metadata once per wizard run, shared by the scope options and the dynamic registration decision, instead of only before the application step.
- Serialized the client id under `--json` even when left empty, matching the `client-id = ""` placeholder the TOML fragment already carried.

### Fixed

- Fired on-issue error hooks when the local device poll deadline expires (`DeviceCodeExpired` as `expired_token`).
- Made the manual `auth resume` command printed by `auth get` runnable as printed.

  Its state and PKCE verifier are single quoted and attached to their flag with `=`, so a value starting with `-` is not read as a flag and one starting with `~` is not expanded by your shell.

- Trimmed the auth-code resume input.
- Omitted the redirect, state and PKCE bodies from resume errors.

### Removed

- Removed the `auth discover` subcommand: bare `ortie` already runs the same wizard. Run `ortie` (it prompts for the email, server or issuer input).
- Removed hand-entry of OAuth 2.0 endpoints from the wizard, along with the "Enter OAuth 2.0 details manually" pick-list entry.

  The wizard configures only what it can discover; when nothing is found it stops and points at the sample configuration.

- Removed the prompts behind the wizard's custom application entry (client id, secret, scopes, redirection).

  If you registered an application yourself you are already editing the config, so the wizard emits the account with everything it resolved, leaves `client-id` empty, and explains what to fill in by hand. The storage step still runs, being independent of the application.

## [2.0.0] - 2026-07-17

### Added

- Added the account discovery wizard, run by bare `ortie` (alias of `auth discover`).

  Prompts for an email address, a server or an issuer URI, discovers the reachable OAuth 2.0 grants and prints the pick as a complete `[accounts.<name>]` fragment: valid TOML on stdout (`ortie >> <config>` appends it directly), or a JSON object with `--json`. Grants sharing a flow and endpoints are grouped into a single choice that merges their per-service scopes, so one token can cover several services (Microsoft's IMAP and SMTP, say) instead of one grant line per service.

  The application step then offers every way to obtain a client, most preferred first: dynamic registration (RFC 7591) when the provider advertises it in its RFC 8414 metadata (the wizard registers ortie on the spot, falling back to a reverse-DNS private-use redirection scheme for providers like Fastmail whose registration accepts only those, which `auth get` then completes through a manual `auth resume`), a well-known public application (Thunderbird for Google, Microsoft and Fastmail), or a custom entry.

  It also fills the defaults a provider is known to need but discovery does not surface, such as Fastmail's RFC 8707 `resource` indicator and its scopes (without which Fastmail rejects the authorize request before any consent screen).

  It then lets you pick the scopes in a multi-select (the discovered set, plus any extra the provider is known to advertise), so you can trim or extend what the token is granted.

  The storage step plugs the token storage into a credential provider CLI known for your platform (secret-tool, kwallet-query, security, pass). Ortie never writes the config itself.

- Added the `grant` account config field.

  Selects the OAuth 2.0 grant flow run by the auth commands; defaults to `authorization-code`, the previous implicit behavior.

- Added the `extras` account config table.

  Parameters forwarded verbatim to the authorization request, for provider-specific options like Google's `access_type` and `prompt`, `login_hint`, or the RFC 8707 `resource` indicator.

- Added the token issuance time to the `token inspect` output.

### Changed

- Enabled PKCE by default with the S256 method, aligning with OAuth 2.1.

  The `pkce` config field accepts booleans (`true` = S256, `false` = off) and method strings (`"s256"`, `"plain"`); servers rejecting PKCE parameters need an explicit `pkce = false`.

- Changed the storage and hook command shapes.

  A string command runs through the platform shell with env-var expansion; an exec-style `[program, arg, ...]` array runs directly, without expansion. Configurations relying on env-var expansion inside array commands must switch to the string form.

- Made every `endpoints.*` config field optional.

  Each command checks the endpoints it actually needs, so `token show` works on a minimal account holding only `client-id` and the storage commands.

- Replaced the deprecated `--debug` and `--trace` CLI flags with `--log-level <level>` and `--log-file <path>`.

- Re-licensed the project from AGPL-3.0-only to dual MIT OR Apache-2.0.

### Removed

- Removed the library target.

  Ortie is now a pure CLI binary; library users should depend on [io-oauth](https://github.com/pimalaya/io-oauth) directly.

- Removed every non-TLS cargo feature (`oauth2`, its `rfc6749` alias, `command`, `cli`, `client`).

  The binary always builds with the full CLI included; remaining features are the TLS providers (`rustls-ring` default, `rustls-aws`, `native-tls`), `vendored` and `notify`.

## [1.1.0] - 2026-02-16

### Changed

- Replaced the default TLS feature `native-tls` by `rustls-ring`.

  Native TLS complicates the release process and produces heavier static binaries.

### Removed

- Removed direct keyring support, in favour of storage commands.

  What keyring support did behind the scenes was never clear to users, and it added complexity for it. A credential provider CLI does the same job, the way ortie itself exports the OAuth logic for other tools to call.

## [1.0.0] - 2026-02-12

### Added

- Added support for custom authorization parameters ([#4]).

### Changed

- Changed default cargo features to `native-tls`, `command`, `keyring` and `notify`.
- Made the redirection endpoint optional. If omitted, `http://127.0.0.1:0` is used by default, which will start the redirection server on a random port ([#3]).
- Replaced `on-issue-access-token` by `hooks.on-issue`.
- Replaced `on-refresh-access-token` by `hooks.on-refresh`.

### Fixed

- Fixed release build with `native-tls` and `keyring` features.

## [0.1.0] - 2025-10-24

### Changed

- Init auth and token commands
- Replaced pimalaya tui by toolbox
- Bumped all dependencies

### Fixed

- Fix CI and release builds

[#3]: https://github.com/pimalaya/ortie/issues/3
[#4]: https://github.com/pimalaya/ortie/issues/4

[unreleased]: https://github.com/pimalaya/ortie/compare/v2.2.0...master
[2.2.0]: https://github.com/pimalaya/ortie/compare/v2.1.0...v2.2.0
[2.1.0]: https://github.com/pimalaya/ortie/compare/v2.0.0...v2.1.0
[2.0.0]: https://github.com/pimalaya/ortie/compare/v1.1.0...v2.0.0
[1.1.0]: https://github.com/pimalaya/ortie/compare/v1.0.0...v1.1.0
[1.0.0]: https://github.com/pimalaya/ortie/compare/v0.1.0...v1.0.0
[0.1.0]: https://github.com/pimalaya/ortie/compare/root...v0.1.0
