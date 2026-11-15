---
cairn: delta
id: wizard-no-custom-prompts
---

# Delta

## ADDED Requirements

### Requirement: A custom application is not prompted for
The custom entry SHALL prompt for nothing: not the client id, not its secret, not the redirection endpoint. Registering an application of one's own is the rare path, and whoever took it is already editing the configuration, so typing those fields into a wizard only to check them in a file afterwards helps nobody. The account SHALL be emitted with everything the wizard did resolve (grant, endpoints, discovered scopes, storage) and an empty `client-id`.

An account left without an application SHALL be explained on stderr next to the printed fragment: what is missing, the field to fill in, that a secret and a pinned redirection are added when the provider requires them, and where every field is documented. It SHALL NOT be told to run `ortie auth get`, which cannot succeed until the client id is filled in.

## MODIFIED Requirements

### Requirement: Client source preference order
At the application step the wizard SHALL offer every way to obtain a client, sorted by the io-oauth preference order: dynamic registration (RFC 7591) when the authorization server metadata advertises a `registration_endpoint`, well-known public applications registered against the same authorization server, then a custom entry. The decision reads the metadata probed once for the run rather than probing again. The step SHALL run before the scope step, and hand it the scope options its outcome allows. A pick list holding the custom entry alone SHALL be skipped entirely, that entry asking nothing.

### Requirement: Scopes follow the application
The scope step SHALL run after the application step, because what a token may request is a property of the application carrying it. The application step decides where the options come from:

- a well-known public application offers exactly the scopes its registration is granted, with the discovered ones selected, so a scope its client id was never verified for cannot reach the authorization request;
- dynamic registration prompts from the authorization server's advertised scopes, widened by the discovered ones, from inside the registration branch, since the scopes travel in the registration request;
- a custom application prompts for nothing and leaves the discovered scopes in the fragment, since nothing exposes the scopes tied to a registration the user made by hand.

### Requirement: Complete, paste-ready fragment
The emitted fragment SHALL be runnable as printed whenever the wizard obtained an application: an `[accounts.<name>]` header keyed on the derived account name, the client id and secret obtained at the application step, the resolved grant and endpoints and scopes, `auto-refresh`, and the storage commands. Only the fields the user left for later appear blank: the client id of a custom application, and the storage commands when none were named. There is no issuer placeholder, since an unresolved issuer never reaches the fragment. An empty client id SHALL be emitted as an empty placeholder in both output shapes, so the JSON object carries the same key the TOML fragment does.

## REMOVED Requirements

None.
