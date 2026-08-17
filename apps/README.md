# AWE Native Applications

AWE applications are **user-space programs**, not kernel modules.

## Native format

The planned native package is `*.awe`:

```text
manifest.awe.json
program.awe
assets/
```

The manifest declares ABI version, requested capabilities, memory/CPU quotas,
entry point and required services. `appd` validates the manifest before a
process is created by CellKernel.

## First-class AWE applications

The initial platform should ship with these native apps/services:

- **AWE Shell** — command/session environment.
- **AWE Files** — file manager and storage browser.
- **AWE Terminal** — terminal emulator and scripting frontend.
- **AWE Settings** — hardware, users, permissions and system configuration.
- **AWE Monitor** — processes, driver health, memory, CPU and IPC telemetry.
- **AWE Store** — signed package discovery and installation.
- **AWE Network** — network profiles and connectivity management.
- **AWE DevKit** — compiler/toolchain/debugger frontend for AWE apps.
- **AWE Compatibility Center** — launch Linux/Windows/Android compatibility runtimes.

## Architecture

```text
AWE App
  |
  v
appd / App Runtime
  |
  +---- sandbox + capabilities
  +---- IPC to system services
  +---- IPC to driverd
  |
  v
CellKernel process/address-space primitives
```

No application framework is linked into CellKernel. The kernel provides the
minimum primitives; appd and user-space services provide the product layer.
