---
cairn: change
id: hook-variables-camel-case
status: active
created: 2026-11-05
---

# Hook variables become camelCase at 3.0

## Why

Pimalaya guideline cli-008 gives hooks one shape across products: camelCase variables, as `--json` keys are, handed to a command as environment variables of the same names. ortie's are uppercase snake case (`$EXPIRES_IN`). Its hooks run a command only since dependencies-trimmed (2026-11-09) removed the `notify` tables, so no notification template is involved. The 3.0.0 that json-keys-are-camel-case already forces is where the rename costs nothing more.

## What

- **Names**: `ACCESS_TOKEN` to `accessToken`, `TOKEN_TYPE` to `tokenType`, `EXPIRES_IN` to `expiresIn`, `REFRESH_TOKEN` to `refreshToken`, `SCOPE` to `scope`, `ERROR` to `error`, `ERROR_DESCRIPTION` to `errorDescription`, `ERROR_URI` to `errorUri`.
- **Commands keep the former names too, until 4.0**: a command gets both spellings in its environment. A command cannot be checked, and a former name gone missing reads as an empty string: `pass insert` fed `$ACCESS_TOKEN` would store an empty token without a word. The twin names cost nothing to export and stop that; the CHANGELOG and the sample say which are former.
- `${VAR:-default}` keeps working, the command being run by the shell.

## Not done

- Dropping the former command variables: at 4.0, listed in the CHANGELOG then.
