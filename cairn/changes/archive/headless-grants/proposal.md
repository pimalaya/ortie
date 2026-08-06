---
cairn: change
id: headless-grants
status: landed
created: 2026-08-06
---

# Headless client credentials grants

## Why
Ortie is becoming the single OAuth home for the Pimalaya gateway composition: sync tools (neverest) stay OAuth-ignorant and exec an ortie command for a guaranteed-fresh bearer token. The grants they need are the fully headless ones: the client credentials grant (RFC 6749 section 4.4) and its JWT-assertion-authenticated flavor (RFC 7523 section 2.2, Microsoft certificate credentials). io-oauth master already ships both exchanges plus the assertion signer behind the `jwt-bearer` feature; the gap is Ortie's config field and CLI wiring, exactly like the device grant before it.

## What
Two new values on the flat `grant` selector: `client-credentials` (client id, `endpoints.token`, `scopes`, `client-secret` through the existing secret shapes) and `client-credentials-jwt` (client id, `endpoints.token`, `scopes`, `client-key` as a path to a PKCS#8 or PKCS#1 PEM private key, optional `client-certificate` as a path to a PEM or DER certificate for the `x5t` thumbprint Microsoft requires). `auth get` runs the exchange headlessly in one shot; `auth resume` has nothing to resume and says so.

Neither grant issues a refresh token, so the auto-refresh path branches per grant: refresh-token POST where one exists, silent re-acquisition (re-run of the grant) for both client credentials kinds, on `token show --auto-refresh` and `token refresh` alike. Each JWT re-acquisition mints a fresh assertion (new `iat`/`exp` on a short validity, unique `jti`, `x5t` recomputed from the certificate, key re-read from disk); assertions are never stored, and the token store keeps only what the server returns (access token plus expiry). A provider `invalid_client` on the JWT kind carries a hint that the certificate credential likely needs renewal.

## Deferred: the JWT bearer authorization grant (RFC 7523 section 2.1)
The Google service account flavor uses the assertion as the grant itself, with the issuer being the service account email, an impersonated `sub` user and the scopes inside the claims. That is a different claim-set semantic needing its own config fields (issuer and subject no longer equal the client id), so it does not drop into the client credentials machinery unchanged and stays out of this change. io-oauth already ships the exchange (`request_jwt_bearer_grant`) when a future change picks it up.

## Deferred: command-sourced private keys
`client-secret` reuses the pimalaya-config secret shapes, but the private key cannot: the secret command resolver reads a single output line, and PEM documents are multi-line. The key therefore stays a file path, re-read at every mint. A multi-line secret source can lift this later without a config break (the path field remains).
