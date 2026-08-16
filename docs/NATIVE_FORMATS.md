# AWE native formats

## `.asd` — AWE System Driver

The ASD container is the native driver delivery unit. It is parsed without
heap allocation and without external crates by the kernel. The fixed header
contains version, ABI, metadata length, payload length, signature length and
entry offset. Metadata is bounded to 4096 bytes.

A driver is **not trusted because it is an ASD file**. The future load pipeline
must enforce: architecture/ABI match, driver identity, dependency resolution,
capability policy, signature verification, memory permissions, isolation policy,
and device binding before execution.

## `.awos` — AWE native application

AWOS v1 contains a fixed header followed by manifest, code, data and an
optional detached signature. Manifest is bounded to 8192 bytes. The kernel
parser only exposes validated byte ranges; execution and policy remain separate.

The AWEOSA builder can create an unsigned development package. Release images
must pass the signing and policy pipeline before installation.

## Compatibility rule

Linux, Windows and Android applications/drivers are compatibility inputs, not
native AWE binaries. They must pass through explicit ABI/translation layers and
never bypass AWE capability and security policy.
