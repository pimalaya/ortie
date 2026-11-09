---
cairn: log
change: getrandom-duplicate-shown
landed: 2026-11-09
---

# getrandom duplicate shown

deny.toml no longer skips getrandom 0.2: `cargo deny check` warns about the duplicate until ring publishes the 0.17 release its main branch holds since 2026-03-09 (getrandom 0.4.2, c8db6861; release asked in ring#2840), and `cargo update -p ring` then clears it. The RUSTSEC-2023-0071 note is two lines, and the targets list carries no comment.

Capabilities moved: packaging (cairn/spec/packaging.md: "Dependency checks pass clean" modified).
