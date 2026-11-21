---
cairn: tasks
change: the-grant-refusals-exit-2
---

- [x] src/error.rs: `usage`
- [x] src/auth.rs, src/auth/listen.rs, src/auth/resume.rs: the refusals through `error::usage`
- [x] src/mcp.rs: `auth_resume` refusing an unparsable URI coded `usage-invalid`, the argument refusals through `error::usage`
- [x] tests/auth_listen.rs: `a_command_line_the_grant_refuses_exits_2_coded` (failed on 801e5dd), the two listen refusal tests moved to exit 2
- [x] tests/mcp.rs: `a_mis_pasted_uri_leaves_the_flow_pending` asserts `usage-invalid` (failed on 801e5dd)
- [x] spec auth.md, output.md, mcp.md; log
- [ ] CHANGELOG lines (handed to the owner, the file being rewritten elsewhere)
