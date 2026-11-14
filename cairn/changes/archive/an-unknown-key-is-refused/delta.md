---
cairn: delta
change: an-unknown-key-is-refused
---

# Delta

## ADDED Requirements

### Requirement: The configuration refuses every key it does not read
The configuration SHALL refuse at load, naming it, every key it does not read, at every level: the document root, which holds `accounts` alone; an account; and every table nested in one (`endpoints`, `storage`, `storage.read`, `storage.write`, `client-secret`). `extras` alone takes any key, its keys being wire parameter names. The `hooks` table, which Ortie read until 3.0.0, SHALL be refused by name whatever it holds, naming `storage.write.command` as its replacement and linking the CHANGELOG.

#### Scenario: A typo in a nested table
- GIVEN an account carrying `endpoints.tokn = "https://login.example.com/token"`
- WHEN any command loads the configuration
- THEN it fails naming the unknown field `tokn`

#### Scenario: A 2.x hook
- GIVEN an account carrying `hooks.on-issue.success.command = "notify-send Ortie"`
- WHEN any command loads the configuration
- THEN it fails saying the `hooks` table was removed in 3.0.0, naming `storage.write.command` and https://github.com/pimalaya/ortie/blob/master/CHANGELOG.md

## MODIFIED Requirements

### Requirement: v1 config compatibility
Every v1.x config file SHALL parse and run unchanged, the `hooks` table aside, refused by name ("The configuration refuses every key it does not read"): `grant` defaults to authorization-code, `endpoints.authorization` merely becomes optional, and `pkce = true`/`false` are still accepted. The behaviours an existing account can notice are PKCE by default and `token show` refreshing an expired token, unless it sets `auto-refresh = false`.
