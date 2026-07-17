# Future Work

## Overview

This document outlines planned features and improvements for future versions of RuntimeShield.

## Priority Features

### Windows Platform Support

Full Windows implementation is the highest priority:

- **Process identity**: `GetCurrentProcessId`, `GetModuleFileName`, `NtQueryInformationProcess`
- **Debugger detection**: `IsDebuggerPresent`, `NtQueryInformationProcess(ProcessDebugPort)`, `CheckRemoteDebuggerPresent`
- **Memory regions**: `VirtualQueryEx`, `ReadProcessMemory`
- **Library enumeration**: `EnumProcessModules`, `GetModuleFileNameEx`

Windows-specific dependency: `windows-sys` or `winapi`

### macOS Feature Completion

- **Debugger detection**: Implement `sysctl` P_TRACED check using `libc` crate
- **Library enumeration**: Use `_dyld_get_image_name` and `_dyld_get_image_header`
- **Memory regions**: Implement `mach_vm_region` and `mach_vm_read` for memory integrity

## Additional Integrity Modules

### Integrity Measurement Architecture (IMA) Integration

For Linux systems with IMA enabled, integrate with the kernel's integrity measurement system for enhanced verification.

### Code Signing Verification

Verify application code signatures before execution:

- Linux: `authenticode`-style verification (debatable utility)
- macOS: `SecStaticCodeCheckValidity` for Gatekeeper verification
- Windows: `WinVerifyTrust` for Authenticode signatures

### HMAC-Based Integrity

Support HMAC-SHA256 for integrity checks with a secret key, making it harder for attackers to forge valid manifests.

## Advanced Features

### Encrypted Manifests

Support manifest encryption so that expected hashes are not visible in plain text.

### Remote Verification

Forward integrity events to a remote monitoring service for centralized alerting.

### Integrity Dashboard

A reference dashboard application for monitoring RuntimeShield-protected applications.

### Self-Check Module

Periodically verify the integrity of RuntimeShield's own code and configuration.

### Multi-Process Coordination

Support for protecting multi-process applications with coordinated integrity verification.

## Performance Improvements

### Parallel Hashing

Use thread pools for parallel hashing of large binaries to reduce startup time.

### Incremental Verification

Cache and reuse hash computations that haven't changed between verification cycles.

### Memory-Mapped I/O

Use `mmap` instead of `read` for file access to reduce memory copies.

## Non-Technical

### C Bindings

Provide C bindings for integration with non-Rust applications:

```c
#include <runtimeshield.h>

int main() {
    rt_shield_t* shield = rt_shield_new();
    rt_shield_enable_binary_integrity(shield);
    rt_shield_start(shield);
    // ...
    rt_shield_free(shield);
    return 0;
}
```

### Python Bindings

Provide Python bindings for script-level integration via PyO3.

### Benchmark Suite

Expand the benchmark suite with:

- Cross-platform performance comparisons
- Binary size scaling tests
- Memory usage profiling
- Thread-safety stress tests

## How to Contribute

Contributions in these areas are welcome:

1. Review the architecture and design documents
2. Open an issue describing your intended changes
3. Submit a pull request with:
   - Code changes
   - Updated tests
   - Documentation updates

See the repository's CONTRIBUTING.md for detailed guidelines.
