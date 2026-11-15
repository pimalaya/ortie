---
cairn: tasks
change: release-polish
---

- [x] Audit the write-after-refresh window in `token refresh` (rotated token persisted before old one lost); the failing-write gap is closed for a session and named in the error
- [x] README section on the 2.1 posture (grants, PKCE default, exact redirect matching, rotation)
- [x] CHANGELOG notes (pkce default flip, discover output shape, lib removal): already in the 2.0.0 and 2.1.0 entries
- [x] Re-check the final RFC if published; release the minor: not published as of 2026-11-03, and the minor shipped as 2.1.0
