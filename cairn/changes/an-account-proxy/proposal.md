---
cairn: change
id: an-account-proxy
status: active
created: 2026-11-19
---

# An account proxy

## Why

Finding ORT-10 of the CLI family analysis of 2026-11-15 (LOW, cohesion; audit finding F40 before it). himalaya and neverest read a `proxy.*` table per account; Ortie reads none, and io-oauth 0.7.0 connects with pimalaya-stream's default `Proxy::System`, the proxy of the environment. A user who sets an account proxy in himalaya for privacy still has every token refresh of that account, run by Ortie as its secret command, leave directly. The CLI alignment ledger's answer ("L22 does not apply: ortie reads no proxy key, its OAuth client honours the proxy environment") does not cover that case.

## What (recommendation, not implemented)

1. io-oauth: `Oauth20ClientStd` connects through a `pimalaya_stream::proxy::Proxy` the caller gives (a `proxy` field on its connect options, or a `connect` taking `TlsConnectOptions` and `TcpConnectOptions`), `Proxy::System` staying the default. A release of io-oauth, outside this repository.
2. Ortie: an optional `proxy` table on the account, the family's shape (`ProxyConfig`, the COH-6 table, copied five times today and due to move into pimalaya-config), passed to every connection: grants, refresh, revocation, device polling and the `account check` probes.
3. The sample and the spec say that an application calling `ortie token show` does not pass its own proxy: the account in Ortie's configuration carries it.

Additive, no deadline; it waits for io-oauth and preferably for COH-6's move of `ProxyConfig` into pimalaya-config.
