---
cairn: spec
capability: packaging
status: current
---

# Packaging

Ortie is a thin, config-driven front-end over io-oauth, the OAuth engine. It ships as a binary only, with no library surface, and adopts OAuth 2.1-compatible defaults without modelling a protocol version.

### Requirement: Pure CLI binary
Ortie SHALL build as a binary with no public library API. The lib target and the `cli`/`client` cargo features do not exist; the README and CHANGELOG redirect the rare library user to io-oauth.

### Requirement: TLS-only feature set
The cargo features SHALL be the TLS providers (rustls-ring by default) and `vendored`. The non-TLS features (`oauth2`/`rfc6749`, `command`, `cli`, `client`, `notify`) are dropped.

### Requirement: No version switch
Ortie SHALL NOT expose a `version` config option for OAuth 2.0 versus 2.1. 2.1 is a constraint profile of 2.0 with identical wire messages; Ortie adopts 2.1-compatible defaults (S256 PKCE, exact redirect matching, refresh rotation) with explicit 2.0 escape hatches (`pkce = false`, `pkce = "plain"`). A hard-strict mode, if ever needed, would land later as a single `strict = true` flag.

### Requirement: Licence
Ortie SHALL be dual-licensed MIT OR Apache-2.0.

### Requirement: Hosts are ASCII
Ortie SHALL build url's IDNA without a Unicode back end, `idna_adapter` being pinned to 1.0.0 in Cargo.toml. A host SHALL be written in ASCII, an internationalised one in its Punycode form (`xn--…`), and a URL with a non-ASCII host SHALL be refused when parsed.

#### Scenario: An internationalised token endpoint
- **GIVEN** `endpoints.token = "https://bücher.example/token"`
- **WHEN** the configuration is read
- **THEN** it is refused, and the same account is read with `endpoints.token = "https://xn--bcher-kva.example/token"`

### Requirement: Dependencies name their features
Every dependency SHALL be declared with `default-features = false` and the features Ortie uses, so a default turned on upstream never reaches the binary. clap SHALL be built without `color` and `wrap_help`, and pimalaya-cli without env_logger's `auto-color`: help, usage and logs carry no colour, nothing asks the terminal for its size, and help prints each paragraph on one line.

### Requirement: Dependency checks pass clean
`cargo deny check` SHALL pass over the targets deny.toml lists, which are the release targets and the ones people build from source. A duplicate the tree cannot avoid stays a visible warning rather than a skip: today getrandom 0.2, which ring 0.17 needs until its next release.
