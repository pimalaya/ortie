---
cairn: delta
id: the-contract-audit-rulings
---

# Delta

## ADDED Requirements

None.

## MODIFIED Requirements

### Requirement: v1 config compatibility (config)
The `hooks` table and the `tls` string are refused as any key Ortie does not read or value of the wrong type, no longer by name.

### Requirement: One TLS conversion, taking the ALPN list (config)
The table MAY hold `client.cert` and `client.key`, both required once `client` is given, the keys every Pimalaya CLI spells alike.

### Requirement: A bare invocation shows the help (config)
A bare `ortie` answers through pimalaya-cli's `clap::meet_bare_invocation`: under `--json` it fails coded `usage-invalid`, exit 2; in text it shows the help, or the missing configuration.

### Requirement: The configuration refuses every key it does not read (config)
`tls.client` is refused an unknown key too. A key an earlier release read is refused by serde alone, never by name: the scenarios *A 2.x hook* and *A 2.x TLS string* now expect serde's unknown field and wrong type.

### Requirement: The check proves the storage reads and the endpoints answer (account)
The output is a list of one account, `{"accounts": [{"account", "backends"}]}`, each row carrying `tls`: `null` on `storage`, on `endpoints` the TLS configured once one checked endpoint is `https`. New scenario *Endpoints over TLS*.

### Requirement: the handoff (auth)
`--handoff` on a grant without one is refused coded `usage-invalid`, exit 2, and `--handoff -` inside a session through `session::check_stdin`, coded `stdin-taken`.

### Requirement: `auth listen` (auth)
With neither `--redirect-uri` nor `endpoints.redirection`, it is refused coded `usage-invalid`, exit 2.

### Requirement: A failure carries a stable code (output)
`usage-invalid` covers the lines Ortie refuses after parsing them; `stdin-taken` joins the table.

## REMOVED Requirements

None.
