---
cairn: delta
change: config-conventions
---

# Delta

## ADDED Requirements

### Requirement: One TLS conversion, taking the ALPN list
The `tls` account option SHALL name the TLS provider as one of `auto` (the default, the provider the binary was built with), `native-tls`, `rustls-aws` or `rustls-ring`, and SHALL be folded into the runtime TLS handle by a single conversion taking the ALPN list as its argument. No other code path SHALL build that handle from the configuration, so no call site can negotiate an ALPN, or none, by accident.

#### Scenario: Omitted TLS provider
- GIVEN an account with no `tls` field
- WHEN a command connects to the token endpoint
- THEN the provider is the one the binary was built with

### Requirement: ALPN is a config key
An account MAY declare `alpn`, the list of ALPN identifiers offered during the TLS handshake. It SHALL default to empty, sending no ALPN extension, which is what an OAuth 2.0 endpoint over plain HTTPS expects; a non-empty list overrides it, `["http/1.1"]` being the one meaningful value, for a TLS middlebox refusing a handshake without ALPN. Only rustls reads it, native-tls ignoring ALPN.

### Requirement: A defaulted field is absent from the generated account
The account fragment the wizard prints SHALL omit every field equal to its type's default, through one shared helper rather than a per-field predicate, so an account that does not claim the default carries no `default = false` line in either rendering.

## MODIFIED Requirements

## REMOVED Requirements
