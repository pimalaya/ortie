---
cairn: log
change: adopt-cairn
landed: 2026-07-25
---

# Adopt Cairn

Converted the ad-hoc `docs/` folder (`README.md`, `oauth21-plan.md`, `discovery-layering.md`, `providers.md`) into a Cairn root. The current, landed truth became six spec capabilities (`config`, `token`, `auth`, `discovery`, `packaging`, `provider-quirks`). The plan's "Landed" milestones became dated log entries (M0 pure-cli-binary, M1 config-schema-and-extras, the wizard-default-command and source-tree-refresh refreshes, M5 dynamic-client-registration). The still-in-flight roadmap became change proposals (`device-grant` M2, `release-polish` M3, `discovery-upgrades` M4, `token-revocation` M6, and the cross-crate `discovery-layering` refactor). Defaults apply throughout, so no `cairn.toml` is needed; the `cairn/` directory alone marks the root.

This is a documentation reorganisation with no behaviour change: the spec captures what Ortie already does today, seeded once from the existing design docs.
