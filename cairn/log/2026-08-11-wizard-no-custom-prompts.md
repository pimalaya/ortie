---
cairn: log
id: wizard-no-custom-prompts
date: 2026-08-11
---

# Stop prompting for a custom application

The custom entry of the application step asked for a client id, a secret, a scope string and a redirection endpoint. It asks for nothing now. `custom()` is gone; the entry returns the scope source directly and the account keeps an empty `client-id`.

Everything the wizard actually resolved still lands in the fragment: the grant, its endpoints, the discovered scopes, and the storage commands. Only the application is left blank, and the wizard says so on stderr right after printing the fragment, naming the field to fill in, mentioning `client-secret.raw` and `endpoints.redirection` for providers that need them, and pointing at the sample configuration. Such an account is not told to run `ortie auth get`, which cannot succeed until a client id is there.

The reasoning is the split between the two audiences. The wizard is for the majority who want a working account out of one email address, on a public application or dynamic registration. Whoever registered an application of their own is already in the config file, and typing four values into prompts to review them there afterwards buys nothing. The storage step stays for them all the same: it is orthogonal to the application, and the argv and shell shapes it emits (down to the macOS `$(cat)` quirk) are the part genuinely worth not writing by hand.

A pick list left with the custom entry alone is now skipped rather than shown, since there is nothing to answer.

Capabilities moved: discovery.
