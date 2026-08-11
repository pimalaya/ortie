---
cairn: log
id: wizard-print-then-save
date: 2026-08-11
---

# Print the account, then offer to save it

The wizard prints its fragment on stdout in every mode now, and the save step comes after it. `save_or_print` became `offer_save`, taking the account by reference and no printer, since printing is no longer one of its arms.

That removes the reason the prompt had to explain itself: `Save this configuration to a file (no prints it)?` is now `Save this configuration to a file?`, no simply ending the wizard with the fragment already in front of the user. Choosing where a configuration goes after seeing it is the only order that makes sense.

The append itself is confirmed too when the chosen file already holds something, naming the path: it is the user's config, and writing into it is the one irreversible thing the wizard does. Declining ends the save, the fragment being printed already. A missing or empty file is written without asking, having nothing to preserve.

A redirected stdout and `--json` are unchanged: they print and never prompt.

Capabilities moved: discovery.
