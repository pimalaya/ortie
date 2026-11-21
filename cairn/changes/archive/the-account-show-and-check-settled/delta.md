---
cairn: delta
id: the-account-show-and-check-settled
---

# Delta

## MODIFIED Requirements

### Requirement: The show prints the account as loaded (account)
The text is a plain view, `Account: <name>` then one aligned `<key>: <value>` field per property as pimalaya-cli's `fields::Fields` prints it, labelled by its configuration key, a list one value per line, a command as a shell reads it, every label and value sanitized, a raw secret `client-secret.raw: <redacted>`; it no longer pastes back into a configuration. The JSON leaves `tls` out when the account sets no TLS key. Scenarios: "The defaults filled in" without `tls`, "The plain view" added, "A raw secret" without the paste-back.

### Requirement: The check proves the storage, the token and the endpoints (account)
Each row prints as a `check-finished` event, `{"event", "account", "check", "ok", "code", "error", "tls"}`, as soon as it is checked, then the report as before; text events on a terminal only, the redirected text holding the report alone. Scenario "A reachable account" names the events; "A check redirected to a file" added.

### Requirement: Commands carrying a schema (output)
`ortie-account-check` also carries its events schema, `ortie-account-check.events`, in the registry's `events` map.

### Requirement: Only Ortie's own types are described (output)
`AccountCheckEvent` joins the account outputs owned by Ortie.

### Requirement: Command grammar reuse (repl)
An `account check` line prints its events before its report.
