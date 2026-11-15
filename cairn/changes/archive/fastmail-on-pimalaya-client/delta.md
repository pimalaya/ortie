---
cairn: delta
change: fastmail-on-pimalaya-client
---

## MODIFIED Requirements

### Requirement: Fastmail resource and redirect
The Fastmail authorize endpoint SHALL receive the JMAP session URL as RFC 8707 resource and a non-empty scope, or it bounces pre-consent; the session's RFC 9728 metadata names the resource. It forwards every authorization parameter to a strict check, so none beyond RFC 6749, PKCE, `resource` and `login_hint` is sent. Its dynamic registration answers the same client id to the same request, and since November 2026 accepts loopback redirects: the sample's client `df6e41ae` was registered with `http://127.0.0.1/` and `http://localhost/`, and the listener catches `http://localhost/` on an ephemeral port. Its metadata also advertises client ID metadata documents (`client_id_metadata_document_supported`), unused so far.
