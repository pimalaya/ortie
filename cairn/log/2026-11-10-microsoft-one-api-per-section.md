---
cairn: log
change: microsoft-one-api-per-section
landed: 2026-11-10
---

# Microsoft, one API per section

config.sample.toml's Microsoft section asked `graph.microsoft.com` and `outlook.office.com` scopes together, and its code exchange failed with AADSTS28003 (scope empty), Entra issuing a token for one resource. It is now two sections on Pimalaya's application, "Microsoft Graph" (`Mail.ReadWrite`, `Mail.Send`, `Calendars.ReadWrite`, `Contacts.ReadWrite`) and "Microsoft IMAP and SMTP" (`IMAP.AccessAsUser.All`, `SMTP.Send`), both with `offline_access`. `the_sample_configuration_parses_with_each_variant` names both. The owner signed in with Microsoft once the scopes were split.

Capabilities moved: config (cairn/spec/config.md: "One sample configuration, with a section per provider" names the two Microsoft sections).
