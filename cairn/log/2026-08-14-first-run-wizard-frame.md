---
cairn: log
change: first-run-wizard-frame
landed: 2026-08-14
---

# Met the newcomer the way Comodoro does

Ortie took Comodoro's wizard frame, the third product to. What the wizard produces is untouched: discovery, the application step, the scopes, the storage and the grant test all decide the account exactly as `wizard-himalaya-parity` and its successors left them. What changed is when the wizard is reached at all.

`take_account` used to bail with `Config file not found` and `Account not found`, the two messages Comodoro's log named as the thing to fix. Neither said which path was read, which accounts exist, nor how to pick one, so a mistyped `-c` and a first run produced the same four words. All three failures now name what is missing and what to do about it.

Nothing offered to fix it either: the wizard existed and no command that needed an account ever mentioned it. A missing configuration now raises an offer, from a bare `ortie` and from any command needing an account, and the offer is a hook rather than a gate. The command carries on afterwards either way, so accepting gives it a chance to work and declining leaves it to fail the ordinary way. The configuration is re-read after the wizard rather than assumed, because Ortie's wizard can print the fragment without saving it, so having run it proves nothing landed.

`ortie configure` (alias `wizard`) runs the wizard by name. That frees the bare invocation, which used to run the wizard unconditionally: someone already set up who typed `ortie` to remind themselves of the commands got a wizard instead of the help. A bare `ortie` now offers only when it finds no configuration, and shows the help otherwise, `--account` with no subcommand counting as a half-typed command rather than a first run.

Both entry points check `stdin().is_terminal()` and `printer.is_json()` first, and `configure` refuses outright when it cannot prompt, naming the sample to write by hand.

The welcome moved onto the offer and names the path that was actually looked at. The command asked for by name skips it.

The save stopped prompting for a path and takes it from `Config::target_path`. The generated account now takes a free name, suffixed until it is, and claims `default` only when no other account does, two invariants appending needed and did not have: a duplicate `[accounts.<name>]` makes the whole document unparseable, and a second default makes the account every command picks depend on map ordering. This meant giving the wizard's `OauthConfig` a `default` field, which it did not carry.

`ORTIE_CONFIG` is read like `-c`, and both now split on `:`. This needed clap's `env` feature, which Ortie had not enabled.

Deliberately kept against Comodoro: print-then-save. `wizard-print-then-save` decided the fragment always reaches stdout before the save is offered, because choosing where a configuration goes before having seen it is the wrong order. Comodoro and Himalaya print only as the fallback. Ortie's order is the better one, so the frame adapted to it rather than the reverse: what changed is where the file lands and under what name, not when the fragment is shown. The append confirmation stays too, since the file being appended to is one the user already owns.

Also found while testing: an account fragment carrying no storage does not parse back, `storage` being a required field. The wizard always configures one, so this only shows up in a hand-built fragment, but it is why the new append test has to set it.

Verified: build, fmt and clippy clean; 38 tests pass, four of them new. The non-interactive paths were exercised end to end: a bare invocation with no configuration prints the help without prompting, one with a configuration prints the help rather than the wizard, `configure` refuses and names the sample, and the three resolution failures each print what they promise, including through `ORTIE_CONFIG`.

Spec updated: config (ADDED: A named command runs the wizard, The offer is a hook not a gate, A bare invocation meets the newcomer not the user, The welcome names the missing path, A generated account takes a free name and one default, Account resolution failures name what is missing, The configuration path is read from the environment, The wizard saves where the configuration lives).
