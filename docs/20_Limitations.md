# Limitations

## Overview

RuntimeShield is a user-space integrity verification framework. It has fundamental limitations that you must understand before integrating it into your application.

## Architectural Limitations

### User-Space Only

RuntimeShield operates entirely in user space, with the same privileges as the application. This means:

- **Kernel-level attackers bypass everything**: A rootkit or kernel module can hide processes, modify memory reads, and intercept system calls that RuntimeShield depends on.

- **Same-user attackers**: Any process running under the same user ID can potentially interfere with RuntimeShield.

- **No hardware root of trust**: RuntimeShield does not use TPM, SGX, or other hardware security features.

### Race Conditions

There is an inherent race condition between verification and exploitation:

```
Time:  ├───── Verification ─────┤├───── Execution ─────┤
       │                         ││                      │
       │  Read clean memory      ││  Memory modified     │
       │  Verify: OK             ││  Exploit runs        │
```

The runtime monitor checks at discrete intervals. An attacker can modify state between checks, execute malicious code, and restore original state before the next check.

### Self-Verification Trust Problem

RuntimeShield runs as part of the application it is protecting. If the application binary is modified to remove or disable RuntimeShield, no verification occurs. This is the fundamental "who verifies the verifier" problem.

## Module-Specific Limitations

### Binary Integrity

- **Manifest must be trusted**: If both binary and manifest are replaced, verification is meaningless.
- **Cannot detect in-memory changes**: Only verifies the file on disk.
- **Startup timing**: If the binary is modified and then verified before execution, it's caught. If modified after verification, detection happens at the next check.

### Library Integrity

- **System library updates**: Legitimate system updates change library hashes, causing false positives.
- **LD_PRELOAD bypass**: Libraries injected via LD_PRELOAD before RuntimeShield starts are included in the baseline.
- **Dynamic loading**: Libraries loaded after startup via dlopen() are not verified unless manually checked.

### Memory Integrity

- **Writable data not verified**: Data sections, heaps, stacks, and other writable memory are intentionally excluded.
- **JIT code**: JIT-compiled code is typically in writable pages and is excluded.
- **ASLR**: Addresses change between runs; memory integrity relies on region permissions, not absolute addresses.
- **Self-modifying code**: Applications that legitimately modify their code sections need special handling.

### Anti-Debug

- **Easily bypassed by experienced attackers**: All debugger detection techniques documented in this framework have known bypasses.
- **False positives**: Profiling tools (strace, dtrace, perf) may trigger detection.
- **Compatibility issues**: Some enterprise security software attaches to processes for monitoring.

### Policy Engine

- **`Terminate` action is coarse**: If a violation is detected, `Terminate` exits the process immediately. There is no grace period or cleanup.
- **Policy file must be protected**: If an attacker can modify the policy file, they can disable protections.
- **No dynamic policy updates**: Policies are loaded at startup and cannot be changed at runtime (by design).

## Platform Limitations

### Linux

RuntimeShield depends on `/proc` filesystem availability. Some environments (containers with restricted `/proc`, some sandboxes) may not expose the expected interfaces.

### macOS

Several macOS features are deferred:
- Debugger detection via sysctl
- Library enumeration via dyld APIs
- Memory region reading via mach_vm_region

These will be implemented in future releases.

### Windows

Windows support is currently limited to trait definitions and stub implementations. Full support requires additional platform-specific crates.

## Detection vs Prevention

RuntimeShield is primarily a **detection** framework. It can detect tampering but has limited ability to prevent it:

| Action | Detection | Prevention |
|---|---|---|
| Binary patching | ✅ | ❌ (can Terminate process) |
| Library substitution | ✅ | ❌ |
| Debugger attachment | ✅ | ❌ |
| Memory patching | ✅ | ❌ |
| Kernel rootkit | ❌ | ❌ |

## When Not to Use RuntimeShield

- **As the only security measure**: RuntimeShield should be one layer in a defense-in-depth strategy.

- **In kernel mode**: RuntimeShield is a user-space library. For kernel-level integrity, use IMA, EVM, or similar.

- **As DRM**: RuntimeShield does not control how content is used or copied. Use dedicated DRM solutions.

- **Against nation-state attackers**: RuntimeShield provides no protection against well-resourced, determined adversaries.

- **Without understanding the threat model**: RuntimeShield addresses specific threats. If your threat model is different, the framework may not be suitable.

## Final Note

RuntimeShield is a tool for raising the cost of attacking an application. It makes casual tampering and debugging more difficult, but it does not guarantee security. The only way to make an application completely "unhackable" is to not run it at all.
