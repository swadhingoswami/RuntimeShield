# Introduction

## What is RuntimeShield?

RuntimeShield is a modular runtime protection framework designed for native applications that need to verify their own integrity at startup and during execution. It provides a clean, extensible SDK that developers integrate directly into their applications.

## Design Philosophy

RuntimeShield is built on several core principles:

1. **Framework, not product** — RuntimeShield is an SDK that developers embed in their applications. It is not a standalone security product.

2. **Modular by design** — Every feature is an independent module. Applications enable only the protections they need.

3. **Honest about limitations** — RuntimeShield clearly documents what it can and cannot protect against. It does not claim to make applications "tamper-proof."

4. **Platform-agnostic core** — The core framework is platform-independent. Platform-specific implementations are behind trait abstractions.

5. **Simple API** — The public API is minimal and follows the builder pattern for configuration.

## What RuntimeShield Can Do

| Capability | Description |
|---|---|
| Binary Integrity | Detect modifications to the executable file on disk |
| Library Integrity | Detect modifications to loaded shared libraries |
| Memory Integrity | Detect modifications to executable code sections in memory |
| Debugger Detection | Detect attached debuggers using platform-specific techniques |
| Integrity Events | Notify the application through callbacks when integrity violations are detected |
| Policy Enforcement | Respond to integrity violations according to configurable policies |

## What RuntimeShield Cannot Do

RuntimeShield is not a security panacea. It cannot:

- Prevent determined attackers from reverse engineering the application
- Protect against kernel-level tampering
- Detect all forms of code injection
- Prevent memory dumping
- Protect against hardware-level attacks
- Encrypt or obfuscate the application binary
- Run in ring 0 or provide kernel-level protection

## Target Audience

RuntimeShield is intended for developers of:

- Native desktop and server applications
- Games and game engines (as a complementary integrity check, not anti-cheat)
- Financial trading applications
- Enterprise software that needs runtime integrity verification
- Security-sensitive command-line tools

## How RuntimeShield Differs from...

### Anti-Cheat Systems

Anti-cheat systems (EAC, BattlEye, Vanguard) operate at kernel level with extensive monitoring and aggressive enforcement. RuntimeShield is a user-space framework that applications opt into. It provides integrity verification without the invasiveness and performance impact of anti-cheat systems.

### Antivirus Software

Antivirus software scans for known malware signatures and monitors system-wide behavior. RuntimeShield verifies the integrity of a specific application and its environment.

### DRM Systems

DRM systems restrict how content can be used and copied. RuntimeShield does not enforce usage restrictions. It helps an application detect if its own binary has been modified.

### Linux fs-verity

Linux fs-verity provides kernel-level file integrity verification using Merkle trees. RuntimeShield's binary integrity module implements a similar concept in user space, making it portable across operating systems. See [Binary Integrity](08_Binary_Integrity.md) for a detailed comparison.

## Getting Started

```
runtime_shield = "0.1"
```

Then see the [API Guide](17_API_Guide.md) for usage examples.
