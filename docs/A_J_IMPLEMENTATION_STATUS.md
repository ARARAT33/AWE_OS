# AWE_OS A-J implementation status

This ledger is evidence-oriented. It does not convert contracts or unit tests into release certification.

## This change set

- **A-C:** keeps the existing bounded bring-up/execution primitives and strict validation model; physical CPU activation, page-table activation and context switching still require runtime evidence.
- **D:** `.asd` admission now validates supported production architectures and exposes bounded manifest/payload/signature slices after structural validation.
- **E:** existing GPT and journal paths remain bounded; journal recovery continues to require persistent crash-injection evidence.
- **F:** firewall policy remains deny-by-default and now selects the most-specific matching rule deterministically, rather than first-match ordering.
- **G:** existing init/service manager remains fixed-capacity and fail-closed; userspace device/filesystem/network managers still require full runtime integration.
- **H:** AWOSA now rejects unknown capability bits before I/O admission, strengthening the capability ABI boundary.
- **I:** `.awos` admission now rejects unknown flags and exposes validated bounded package sections for later trust/sandbox services.
- **J:** added `awe-ayui`, a no-std compositor primitive with bounded windows, focus management and FIFO input events. GPU/display backend integration remains outside this contract and still requires runtime evidence.

## Release certification blockers

The master plan defines 100% as a release certification state requiring implementation, tests, runtime/emulator evidence, CI, recovery/error handling and documentation. Remaining mandatory gates include real hardware activation, QEMU/runtime evidence on the current revision, cryptographic signing, persistent storage/network integration, userspace/service integration, AYUI display/input runtime, fuzz/stress, hardware-in-loop validation and signed reproducible release artifacts.

No checkbox in the master plan is changed by this ledger merely because a bounded contract exists.
