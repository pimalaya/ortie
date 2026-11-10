---
cairn: change
id: microsoft-one-api-per-section
status: landed
created: 2026-11-10
---

# Microsoft, one API per section

## Why

The Microsoft section of config.sample.toml asked Graph and Outlook scopes together. Uncommented, `auth get` failed at the code exchange on 2026-11-10 with `AADSTS28003: Provided value for the input parameter scope cannot be empty when requesting an access token using the provided authorization code` (reported by the owner): Entra issues a token for one resource, so it finds no scope to give a code spanning two.

## What

- The Microsoft section splits into "Microsoft Graph" (mail, calendars, contacts) and "Microsoft IMAP and SMTP" (Outlook), each with Pimalaya's application.
- The sample test names both sections.
