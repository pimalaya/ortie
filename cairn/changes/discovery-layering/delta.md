---
cairn: delta
change: discovery-layering
---

## ADDED Requirements

### Requirement: Typed resource wiring
Ortie SHALL set the RFC 8707 `resource` on the grant's request from the discovery-carried `grant.resource` (typed), rather than relying on a hand-written `extras.resource` or a host-match table. The `extras` bag remains for genuinely provider-specific options.
