---
cairn: change
id: wizard-no-custom-prompts
status: landed
created: 2026-08-11
---

# Stop prompting for a custom application

## Why

The custom entry of the application step asks for a client id, a client secret, a scope string and a redirection endpoint. Every one of those is a value the user already holds, in the provider console tab they have open, and is about to check into a configuration file they own.

The wizard exists for the majority who use a public application or dynamic registration and want a working account out of one email address. Whoever registered an application of their own is the minority, and is already editing the config: typing four fields into prompts, only to review them in the file afterwards, buys them nothing. Worse, the prompts imply the wizard needs those values to do its job, when everything it actually resolved (grant, endpoints, scopes, storage) is independent of them.

## What

The custom entry prompts for nothing. It yields the account with an empty `client-id`, keeping everything the wizard did resolve, and the wizard explains on stderr, next to the printed fragment, what to fill in: the client id, plus `client-secret.raw` and `endpoints.redirection` when the provider requires them, pointing at the documented sample.

An account left without an application is not told to run `ortie auth get`, which cannot succeed until the client id is there.

A pick list holding the custom entry alone is skipped entirely rather than shown, since it now asks nothing.

## Non-goals

- Ending the wizard early on the custom entry. The storage step is orthogonal to the application: the read and write commands, their argv and shell shapes and the macOS `$(cat)` quirk are exactly the part worth not writing by hand, whoever registered the application.
- Emitting an empty `client-secret.raw` or `endpoints.redirection`. A public client needs neither, and an empty secret is worse than an absent one: it would be sent as a credential. They are named in the explanation instead.
