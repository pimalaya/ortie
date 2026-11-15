---
cairn: delta
change: family-order-and-defaults
---

# Delta

Each requirement names its capability file in parentheses.

## ADDED Requirements

### Requirement: The command tree follows the family
(packaging) The top-level commands SHALL be `auth`, `token`, `repl`, `mcp` and `account`, then the standard commands in the family's order, `completion`, `manual`, `json-schema`, with pimalaya-cli's abouts. The help of `repl` SHALL open with "Start a session running commands over one account" and name "`quit`, `exit`, or EOF" as its end.

#### Scenario: The help
- GIVEN any configuration
- WHEN `ortie --help` runs
- THEN its commands read `auth`, `token`, `repl`, `mcp`, `account`, `completion`, `manual`, `json-schema`, `help`

## MODIFIED Requirements

### Requirement: TLS-only feature set
(packaging) The cargo features SHALL be the TLS providers (`rustls-ring`, `rustls-aws`, `native-tls`) and `vendored`, and no other, the default set being `rustls-ring` and `vendored`, as in the other Pimalaya CLIs.
