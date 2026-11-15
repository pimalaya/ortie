---
cairn: delta
change: default-is-a-name
---

# Delta

Each requirement names its capability file in parentheses.

## MODIFIED Requirements

### Requirement: Account resolution failures name what is missing
(config) An absent or empty `-a` SHALL take the account marked `default = true`; any other name SHALL take the account of that name, `default` included, which is a name like any other.

Each of the four ways account resolution fails SHALL name what is missing or wrong and what to do about it: a missing configuration names the path it looked for and points at config.sample.toml, a missing named account lists the accounts the configuration does hold, sorted, a missing account named `default` adds "omit `-a` to take the account marked default", a missing default names both ways of picking one, and several accounts marked `default = true`, none named, are refused naming them, sorted, rather than one taken.

#### Scenario: `default` is a name
- GIVEN a configuration whose one account `a` sets `default = true`
- WHEN `ortie -a default token show` runs
- THEN it fails with "Account `default` not found, the configuration holds: a, omit `-a` to take the account marked default", coded `account-missing` under `--json`

### Requirement: A failure carries a stable code
(output) The code is also read from pimalaya-config's account resolution error, and the table gains:

| Code | Raised when |
|---|---|
| `account-missing` | no account has the name `-a` gave, `default` included |
| `account-default-missing` | no account is marked `default = true`, and `-a` named none |
| `account-default-several` | several accounts are marked `default = true`, and `-a` named none |
