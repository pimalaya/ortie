---
cairn: log
change: wizard-append-or-print
landed: 2026-08-29
---

# The wizard's save layer took the family's shape

Every Pimalaya CLI's `configure` wizard is meant to behave identically, himalaya being the reference and cardamum having already followed it. Ortie matched that contract in intent and in most of its parts: it read the existing configuration through `ExistingConfig::read`, derived a free account name rather than prompting for one, and claimed the default only when no other account did. The save layer is where it had drifted, and it is all that moved.

`offer_save` is gone, replaced by `save_or_print`, `append_or_print` and the `print_saved` they share. A missing configuration is offered `Save this account to <path>?`, one already there `Append account `<name>` to <path>?`, and declining either prints the account on stdout to place by hand. Two prompts used to ask that in halves, the first deciding whether to save at all and the second whether an existing file could be appended to, which is two questions for one decision the second one already implied.

The account no longer reaches stdout on its way to the prompt. Printing first was a deliberate decision, `wizard-print-then-save` on 2026-08-11, taken when the prompt still had to explain its own no-arm: `Save this configuration to a file (no prints it)?`. The rest of the family solved that same problem from the other end, by naming one action in the prompt and making declining the thing that prints. Both answers remove the parenthetical; only the family's keeps the terminal path quiet, and it is the one that makes `configure` on a terminal and `configure` into a pipe do visibly different things rather than both emitting a document.

`--json` and a redirected stdout are unchanged: they print the account, touch no file, and return before any prompt. That short-circuit is what `ortie configure > config.toml` has always relied on, and it is now the only place the non-interactive document comes from.

Appending stays a plain text append through `OpenOptions::new().append(true)`, never a parse and re-serialize, which is the whole reason a user's comments, account ordering and formatting survive. The leading newline still separates the two tables and still terminates a file ending without one.

The confirmation now names the account rather than the file alone: `Account `<name>` saved to <path>.`, followed by the `-a <name>` hint when another account holds the default. The name was never asked for, so an account that did not claim the default is otherwise unreachable. The product-specific line under it is unchanged, still naming `ortie auth get` and still only when the account has a client id to authorize with, since one without was already told what to fill in.

Separately, the stdin guard lost its `--json` exemption. The config capability has always said `configure` refuses when stdin is not a terminal, but the guard read `!printer.is_json() && !stdin().is_terminal()`, so a `--json` run in a script walked into the first prompt and failed there instead, with a worse message and no mention of the sample configuration.

Nothing about what the wizard resolves changed, and neither did the rendered account. Its keys stay kebab-case, which is the vocabulary of the file it is pasted into, and the JSON envelope stays as it is: `configure --json` emits exactly the document it emitted before.

Verified: fmt, check, test and clippy clean, 38 unit and 6 integration tests passing. `configure --json` and `configure > /dev/null` both refuse on a non-terminal stdin, immediately, writing no file; with a terminal stdin and a redirected stdout the wizard reaches its first prompt and still writes nothing. The interactive path was not run against the live configuration.

Spec updated: discovery (ADDED: Saves, appends, or prints; MODIFIED: Saves by appending never overwriting, A custom application is not prompted for; REMOVED: Prints, then offers to save), config (MODIFIED: The wizard saves where the configuration lives).
