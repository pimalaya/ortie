---
cairn: change
id: fastmail-on-pimalaya-client
status: landed
created: 2026-11-10
---

# Fastmail on Pimalaya's client

## Why

The Fastmail section of config.sample.toml carried `client-id = ""` and the private-use redirection `org.pimalaya.ortie://redirect`, which no desktop browser hands back to Ortie: `auth get` failed for the owner on 2026-11-10. Fastmail's authorization server metadata, read the same day, now advertises client ID metadata documents and an MCP resource, a hint that it accepts loopback redirects since the July probe that saw them refused.

## What

- The owner registered a client (RFC 7591) with `client_name` "Ortie", the redirects `http://127.0.0.1/` and `http://localhost/`, no client authentication, the authorization code and refresh token grants. Fastmail answered `df6e41ae`, and the same request answers the same id to anyone.
- The Fastmail section takes `client-id = "df6e41ae"` and `endpoints.redirection = "http://localhost/"`, so the listener catches the redirection as for Google and Microsoft.
- The Google section takes the development client the owner signed in with, the production one having been removed (owner, 2026-11-10); its scopes stay.
- The Fastmail provider quirk, the README and the CHANGELOG say so.
