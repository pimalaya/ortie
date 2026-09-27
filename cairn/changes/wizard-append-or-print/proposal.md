---
cairn: change
id: wizard-append-or-print
status: landed
created: 2026-08-29
---

# Take the family's save layer: append or print, in one prompt

## Why

Every Pimalaya CLI's `configure` wizard is meant to behave identically, himalaya being the reference and cardamum having already followed it. Ortie's wizard matches that contract in intent, and matched it already on reading the existing configuration, deriving a free account name, and claiming the default only when no other account does. Its save layer is where it drifted.

Three differences, in order of what a user notices.

The account reaches stdout in every mode, including the terminal one, and the save is offered afterwards. That was a deliberate decision (`wizard-print-then-save`, 2026-08-11) taken when the prompt still had to explain its own no-arm. The rest of the family answered the same problem the other way: the prompt names one action, and declining is what prints. Printing first costs a screenful of TOML in front of every save, and it means the terminal path emits a document on stdout that the JSON path also emits, so `configure` writing to a terminal is neither quiet nor pipeable.

A file already there is confirmed twice: `Save this configuration to <path>?`, then `<path> already exists, append to it?`. The family asks once, and the one question it asks names both the account and the file: `Append account `<name>` to <path>?`.

The save step is one function, `offer_save`, holding the prompt, the create-or-append branch, and the closing message. The family splits it into `save_or_print`, `append_or_print` and `print_saved`, which is what lets the two file states ask their own question and share one confirmation.

Separately, `configure` refuses to run when stdin is not a terminal, as the config capability requires, except under `--json`, where the guard was disabled. The wizard prompts in JSON mode too, so that exemption only postponed the failure to the first prompt, with a worse message.

## What

`offer_save` becomes `save_or_print` and `append_or_print`, sharing `print_saved`, with the reference's wording verbatim: `Save this account to <path>?`, `Append account `<name>` to <path>?`, `Account `<name>` saved to <path>.`, and the `-a <name>` hint when another account holds the default.

The account reaches stdout on three paths and no others: `--json`, a redirected stdout, and a declined offer. The two non-interactive ones return before any file is touched, which is the contract's short-circuit and what keeps `ortie configure > config.toml` working.

Appending stays a plain text append, never a parse and re-serialize, which is what keeps a user's comments, ordering and formatting.

The stdin guard loses its `--json` exemption.

## Scope / non-goals

Nothing about what the wizard resolves: the prompts, discovery, the application, scope and storage steps are untouched.

Nothing about the rendered account. Its keys stay kebab-case, matching the configuration vocabulary it is pasted into, and the JSON envelope stays as it is. `configure --json` emits the same document it did before.

No new secret reaches stdout: the non-interactive paths print exactly what they printed before.
