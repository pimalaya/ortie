---
cairn: log
change: macos-keychain-write-service
landed: 2026-09-27
---

# Named a service in the macOS Keychain write command

The wizard's `security` write command carried `-a <entry>` alone, since `wizard-no-grant-test-named-entry` dropped the `ortie` namespace. `security add-generic-password` requires `-s`, so storing the token failed after authorization succeeded, from v2.1.0 on (#15).

The fix landed in pimalaya-cli 0.2.5: without a namespace, the entry names both the service and the account. The read by account still finds it, and the other providers are untouched. Ortie only picks up the new version.

Spec unchanged: discovery already requires the entry verbatim in both commands, which still holds.
