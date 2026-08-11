---
cairn: delta
id: wizard-grant-selection
---

# Delta

## ADDED Requirements

### Requirement: A single candidate is not a choice
A step whose reduction leaves exactly one candidate SHALL carry it through without prompting. This holds for the grant pick list and for the application pick list alike: a prompt with nothing to arbitrate is noise, and the entry it would have offered is the one the wizard uses.

### Requirement: Scopes follow the application
The scope step SHALL run after the application step, because what a token may request is a property of the application carrying it. The application step decides where the options come from:

- a well-known public application offers exactly the scopes its registration is granted, with the discovered ones selected, so a scope its client id was never verified for cannot reach the authorization request;
- dynamic registration prompts from the authorization server's advertised scopes, widened by the discovered ones, from inside the registration branch, since the scopes travel in the registration request;
- a custom application keeps a free-text prompt seeded with the discovered scopes, since nothing exposes the scopes tied to a registration the user made by hand.

#### Scenario: Thunderbird on Google
- GIVEN a Gmail account and the Thunderbird public application
- WHEN the wizard reaches the scope step
- THEN the options are the three scopes Thunderbird is registered for (`https://mail.google.com/`, `https://www.googleapis.com/auth/carddav`, `https://www.googleapis.com/auth/calendar`)
- AND a People API scope, which that client id is not verified for, is not offered

## MODIFIED Requirements

### Requirement: Grant-based service reduction
Discovery SHALL reduce services to deduplicated OAuth 2.0 grants tagged with the services sharing them, and present the grant as the unit of choice. Two grants SHALL be reduced into one when they are the same flow against the same authorization server, compared by the host of the endpoint that starts the flow rather than by the endpoint URLs: mechanisms disagree on the exact spelling a provider's endpoints are written with, and the pick list must not ask the user to arbitrate between two spellings of one server. The first spelling seen wins the entry's endpoints, which is the highest-priority mechanism's since compose yields its outputs in mechanism-priority order, and the scopes and services of both are merged into it. The entry SHALL be labelled by its flow and the services it authenticates, never by its endpoints.

### Requirement: Issuers resolve to concrete grants
A discovered `OauthIssuer` entry, and a typed issuer URL, SHALL be resolved through the issuer's RFC 8414 metadata into every grant it advertises: the authorization code grant when an authorization endpoint is published, the device authorization grant when a device authorization endpoint is, and both when both are, since RFC 8414 section 2 and RFC 8628 section 4 let a server advertise them side by side and the choice between them belongs to the pick list. Both need the token endpoint, so a document without one advertises nothing. An issuer whose metadata cannot be resolved into any grant SHALL be dropped from the pick list rather than emitted as a bare issuer comment.

### Requirement: Client source preference order
At the application step the wizard SHALL offer every way to obtain a client, sorted by the io-oauth preference order: dynamic registration (RFC 7591) when the authorization server metadata advertises a `registration_endpoint`, well-known public applications registered against the same authorization server, then a custom entry. The decision reads the metadata probed once for the run rather than probing again. The step SHALL run before the scope step, and hand it the scope options its outcome allows.

### Requirement: Complete, paste-ready fragment
The emitted fragment SHALL be runnable as printed: an `[accounts.<name>]` header keyed on the derived account name, the client id and secret obtained at the application step, the resolved grant and endpoints and scopes, `auto-refresh`, and the storage commands. Only the fields the user deliberately left empty at the application or storage step appear blank; there is no issuer placeholder, since an unresolved issuer never reaches the fragment. A client id left empty SHALL be emitted as an empty placeholder in both output shapes, so the JSON object carries the same key the TOML fragment does.

## REMOVED Requirements

None.
