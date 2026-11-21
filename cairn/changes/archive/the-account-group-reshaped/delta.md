---
cairn: delta
id: the-account-group-reshaped
---

# Delta

## ADDED Requirements

### Requirement: The show prints the account as loaded (account)
`ortie account show` prints every property of the account `-a` names, or the default one, as the configuration loads it, reaching nothing; the text is TOML pasting back into a configuration, the JSON `{"account": {"name", ...}}`; a raw secret prints `<redacted>`, or `{"raw": null, "redacted": true}`.

### Requirement: The check reads the token it found (account)
The `token` row of `account check` reads the token the storage returned, sending nothing: missing fails `token-missing`, expired with no way to renew it fails `token-expired`, expired with one passes.

## MODIFIED Requirements

### Requirement: The list reads the configuration only (account)
`account list` prints `{"accounts": [{"name", "default", "grant"}]}`, no longer `backends` and `notBuilt`.

### Requirement: The check proves the storage reads and the endpoints answer (account)
`account check` prints `{"account", "checks": [...]}`, the rows `storage`, `token`, then one per endpoint the grant uses named by its configuration key, each checked on its own, `tls` on `https` endpoint rows only.

### Requirement: Only Ortie's own types are described (output)
The account outputs are Ortie's own; the failure object is the one shared shape left.

### Requirement: Commands carrying a schema (output)
`ortie-account-show` joins the keys.

### Requirement: A failure carries a stable code (output)
`token-expired` joins the codes.

## REMOVED Requirements

None.
