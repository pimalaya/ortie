---
cairn: tasks
change: account-group
---

# Tasks

- [x] src/account.rs: `AccountCommand` (`list` alias `ls`, `check`), the runtime `Account` gains `name`, `read_storage` runs the read command without memoizing
- [x] src/account/list.rs: `AccountListOutput` from the configuration alone, sorted, empty `backends` and `notBuilt`
- [x] src/account/check.rs: the `storage` and `endpoints` rows, the `HEAD` over `Oauth20ClientStd::connect`, exit 3 on a failed row
- [x] src/cli.rs: `Command::Account`; src/json_schema.rs: `ortie-account-list`, `ortie-account-check`
- [x] tests/account.rs: the list, a passing check against a local listener (the exact request heads, the write command never run), a failed row exiting 3 with both rows, `endpoint-missing` before any request, a non-HTTP answer, `account-missing`
- [x] Spec: account (new), output; README, CHANGELOG, MIGRATION.md (New commands)
