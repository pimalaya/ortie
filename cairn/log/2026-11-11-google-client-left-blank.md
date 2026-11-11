---
cairn: log
id: google-client-left-blank
date: 2026-11-11
---

# Google client left blank in the sample

The Google section of config.sample.toml carries `#client-id = ""` and `#client-secret.raw = ""` instead of the development client of Pimalaya's Google project. Ortie only forwards the scopes an application asks for, so the owner releases Pimalaya's Google Desktop client once Himalaya, Cardamum, Calendula and Unfog are released, with one video covering Ortie and them across the whole scope range. Until then a Google account takes a client of the user's own. The endpoints, `extras.access_type` and the scopes stay. The README feature line and the Unreleased entry on config.sample.toml say so.

Capabilities moved: config (cairn/spec/config.md: "One sample configuration, with a section per provider" names the empty Google client), provider-quirks (cairn/spec/provider-quirks.md: "Current Google endpoints" says an account takes a client of the user's own until Pimalaya's is released).
