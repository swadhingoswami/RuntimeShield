# RuntimeShield

**Cross-Platform Runtime Protection Framework for Native Applications**

[![CI](https://github.com/runtimeshield/runtimeshield/actions/workflows/ci.yml/badge.svg)](https://github.com/runtimeshield/runtimeshield/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/runtimeshield.svg)](https://crates.io/crates/runtimeshield)
[![Documentation](https://docs.rs/runtimeshield/badge.svg)](https://docs.rs/runtimeshield)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE)

---

## Overview

RuntimeShield is a modular runtime protection framework that helps native applications verify their own integrity, detect common runtime tampering techniques, and respond through configurable policies.

**RuntimeShield is NOT:**
- An antivirus
- An anti-cheat system
- DRM
- Malware
- A tool to make applications "impossible" to reverse engineer

**Instead, it provides:**
- Binary integrity verification using Merkle tree-based manifests
- Loaded library integrity checking
- Memory integrity verification for executable code sections
- Anti-debug detection via platform-specific APIs
- Configurable policy engine (Terminate, Callback, Log, Ignore)
- Event callback system for application integration
- Runtime monitoring thread with configurable intervals
- On-demand verification API

## Quick Start

```rust
use runtimeshield::RuntimeShield;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut shield = RuntimeShield::builder()
        .enable_startup_verification()
        .enable_runtime_monitor()
        .enable_binary_integrity()
        .enable_library_integrity()
        .enable_anti_debug()
        .monitor_interval(5000)
        .on_event(Arc::new(|event| {
            println!("{:?}", event);
        }))
        .build()?;

    shield.start()?;

    // On-demand verification
    let result = shield.verify_now()?;
    println!("Integrity OK: {}", result.is_integrity_ok());

    // Keep application running
    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
```

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                     Application                          │
├─────────────────────────────────────────────────────────┤
│                    RuntimeShield API                      │
├──────────┬──────────┬──────────┬──────────┬──────────────┤
│ Builder  │  Config  │  Events  │  Policy  │  Monitor     │
├──────────┴──────────┴──────────┴──────────┴──────────────┤
│                    Integrity Layer                         │
├──────────┬──────────┬──────────┬─────────────────────────┤
│ Binary   │ Library  │ Memory   │  Crypto (Merkle/SHA256) │
├──────────┴──────────┴──────────┴─────────────────────────┤
│                Platform Abstraction Layer                  │
├──────────┬──────────┬────────────────────────────────────┤
│ Linux    │ macOS    │  Windows (designed, deferred)       │
└──────────┴──────────┴────────────────────────────────────┘
```

## Features

| Feature | Description |
|---|---|
| Startup Verification | Verify application integrity before normal execution |
| Runtime Monitor | Periodic verification thread with configurable interval |
| On-Demand Verification | Application can request verification at any time |
| Binary Integrity | Merkle tree-based integrity verification |
| Library Integrity | Verify loaded shared libraries against manifest |
| Memory Integrity | Verify executable code section integrity |
| Anti-Debug | Platform-specific debugger detection |
| Policy Engine | Configurable policies for each event type |
| Event System | Application callback registration |

## Platform Support

| Platform | Status |
|---|---|
| Linux | ✅ Full support |
| macOS | ✅ Full support |
| Windows | 🚧 Architecture designed, implementation deferred |

## Documentation

- [Introduction](docs/01_Introduction.md)
- [Threat Model](docs/02_Threat_Model.md)
- [Architecture](docs/03_Architecture.md)
- [Runtime Protection](docs/04_Runtime_Protection.md)
- [Startup Verification](docs/05_Startup_Verification.md)
- [Runtime Verification](docs/06_Runtime_Verification.md)
- [On-Demand Verification](docs/07_On_Demand_Verification.md)
- [Binary Integrity](docs/08_Binary_Integrity.md)
- [Merkle Tree](docs/09_Merkle_Tree.md)
- [Library Verification](docs/10_Library_Verification.md)
- [Process Identity](docs/11_Process_Identity.md)
- [Memory Integrity](docs/12_Memory_Integrity.md)
- [Anti-Debug](docs/13_Anti_Debug.md)
- [Policy Engine](docs/14_Policy_Engine.md)
- [Event System](docs/15_Event_System.md)
- [Cross-Platform Architecture](docs/16_Cross_Platform_Architecture.md)
- [API Guide](docs/17_API_Guide.md)
- [Examples](docs/18_Examples.md)
- [Performance](docs/19_Performance.md)
- [Limitations](docs/20_Limitations.md)
- [Future Work](docs/21_Future_Work.md)

## License

Licensed under either of:

- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.
