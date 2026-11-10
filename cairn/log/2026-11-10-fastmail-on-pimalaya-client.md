---
cairn: log
change: fastmail-on-pimalaya-client
landed: 2026-11-10
---

# Fastmail on Pimalaya's client

Fastmail's metadata (https://api.fastmail.com/.well-known/oauth-authorization-server, read 2026-11-10) lists `registration_endpoint`, `client_id_metadata_document_supported: true`, the authorization code and refresh token grants only (no device grant), `none` client authentication, S256, and protected resources including its MCP server. The owner then registered `{"client_name": "Ortie", "redirect_uris": ["http://127.0.0.1/", "http://localhost/"], "token_endpoint_auth_method": "none", "grant_types": ["authorization_code", "refresh_token"], "response_types": ["code"]}` at https://api.fastmail.com/oauth/register, which answered `df6e41ae`: loopback redirects, refused in July, are accepted now. With it and `endpoints.redirection = "http://localhost/"`, `auth get` signed in through the loopback listener, as did Google and both Microsoft sections, all from the owner's ~/.ortierc.

config.sample.toml's Fastmail section takes that client and redirection in place of the empty client id and `org.pimalaya.ortie://redirect`. The Google section takes the development client the owner signed in with, the production one having been removed, its scopes unchanged.

Capabilities moved: provider-quirks (cairn/spec/provider-quirks.md: "Fastmail resource and redirect" names the loopback client and the metadata documents).
