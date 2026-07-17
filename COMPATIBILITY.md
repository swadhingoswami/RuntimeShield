# Platform & Compatibility Matrix

RuntimeShield does **not** depend on Linux fs-verity, Windows code integrity, or any OS-specific kernel feature. It uses only standard user-space APIs — `read()`, `/proc/self/*`, `sysctl`, and libc functions — that are available across **all** operating systems, distributions, versions, and filesystems.

---

## Contrast with Linux fs-verity

| Capability | Linux fs-verity | RuntimeShield (user-space) |
|---|---|---|
| **Windows** | ❌ Not available | ✅ Works via std::fs::read |
| **macOS** | ❌ Not available | ✅ Works via std::fs::read |
| **Linux < 5.4** | ❌ Not available | ✅ Works on any kernel version |
| **ext4** | ✅ Supported | ✅ Works |
| **XFS** | ❌ Not supported | ✅ Works |
| **Btrfs** | ❌ Not supported | ✅ Works |
| **ZFS** | ❌ Not supported | ✅ Works |
| **NTFS** | ❌ Not available | ✅ Works |
| **APFS** | ❌ Not available | ✅ Works |
| **tmpfs / ramfs** | ❌ Not supported | ✅ Works |
| **SquashFS** | ❌ Not supported | ✅ Works |
| **FUSE** | ❌ Not supported | ✅ Works |
| **NFS / CIFS** | ❌ Not supported | ✅ Works |
| **SELinux enforcing** | ✅ Works | ✅ Works |
| **AppArmor** | ✅ Works | ✅ Works |
| **Root required** | ✅ Required | ❌ Not needed |
| **Requires kernel module** | ✅ Yes (built-in) | ❌ No — pure user-space |
| **Reads file bytes** | ✅ Kernel handles it | ✅ Application reads them |

**RuntimeShield works everywhere fs-verity works, and everywhere fs-verity doesn't work.**

---

## Operating System Support

| OS | Versions | Architecture | Status |
|---|---|---|---|
| **Ubuntu** | 18.04+, all LTS | x86_64, aarch64 | ✅ Full |
| **Debian** | 10 (Buster)+ | x86_64, aarch64 | ✅ Full |
| **Fedora** | 36+ | x86_64, aarch64 | ✅ Full |
| **RHEL / CentOS** | 8+ | x86_64, aarch64 | ✅ Full |
| **Arch Linux** | Rolling | x86_64, aarch64 | ✅ Full |
| **Alpine Linux** | 3.15+ | x86_64, aarch64 | ✅ Full |
| **openSUSE** | 15.4+ | x86_64, aarch64 | ✅ Full |
| **macOS** | 11 (Big Sur)+ | x86_64, arm64 (M1/M2/M3) | ✅ Full |
| **macOS** | 10.15 (Catalina) | x86_64 | ✅ Full |
| **Windows** | 10 / 11 | x86_64 | 🚧 Stub (design done) |
| **Docker** | All versions | All architectures | ✅ Full |
| **WSL2** | Ubuntu/Debian on Windows | x86_64, aarch64 | ✅ Full |

### Why every distro works

Each check uses only APIs that are guaranteed on every Linux distribution:

| Check | API Used | Availability |
|---|---|---|
| Binary Integrity | `read()` via `std::fs::read` | Every POSIX system |
| Library Enumeration | `read("/proc/self/maps")` | Every Linux kernel with procfs |
| Library Hashing | `read()` via `std::fs::read` | Every POSIX system |
| Memory Code Regions | `read("/proc/self/maps")` + `read("/proc/self/mem")` | Every Linux kernel with procfs |
| Anti-Debug | `read("/proc/self/status")` → parse `TracerPid` | Every Linux kernel with procfs |
| Process Identity | `read("/proc/self/status")`, `readlink("/proc/self/exe")` | Every Linux kernel with procfs |

No distro-specific packages, kernel modules, or security policy exemptions are needed.

---

## SELinux & Mandatory Access Control

| Security System | Binary Integrity | Library Check | Memory Check | Anti-Debug |
|---|---|---|---|---|
| **SELinux (enforcing)** | ✅ Allowed | ✅ Allowed | ✅ Allowed | ✅ Allowed |
| **AppArmor** | ✅ Allowed | ✅ Allowed | ✅ Allowed | ✅ Allowed |
| **Tomoyo** | ✅ Allowed | ✅ Allowed | ✅ Allowed | ✅ Allowed |
| **grsecurity** | ✅ Allowed | ✅ Allowed | ✅ Allowed | ✅ Allowed |

### Why SELinux doesn't block RuntimeShield

RuntimeShield only reads:
1. Its own binary file — SELinux `file:read` permission on `usr_t` or equivalent
2. `/proc/self/*` — SELinux allows a process to read its own `/proc` entries via `proc_type:read`
3. Its own memory via `/proc/self/mem` — SELinux allows self-ptrace via `ptrace:read`

All of these are in the default SELinux policy's allowed set for unconfined and confined domains. No policy module or boolean change is required.

---

## Filesystem Support

| Filesystem | Binary Integrity | Merkle Page Check | Library Hashing |
|---|---|---|---|
| **ext4** | ✅ | ✅ | ✅ |
| **XFS** | ✅ | ✅ | ✅ |
| **Btrfs** | ✅ | ✅ | ✅ |
| **ZFS** | ✅ | ✅ | ✅ |
| **NTFS (via ntfs-3g)** | ✅ | ✅ | ✅ |
| **FAT32 / exFAT** | ✅ | ✅ | ✅ |
| **APFS** (macOS) | ✅ | ✅ | ✅ |
| **HFS+** (macOS) | ✅ | ✅ | ✅ |
| **SquashFS** (read-only) | ✅ | ✅ | ✅ |
| **tmpfs** (RAM disk, Docker layers) | ✅ | ✅ | ✅ |
| **ramfs** | ✅ | ✅ | ✅ |
| **NFS v3/v4** | ✅ | ✅ | ✅ |
| **CIFS / SMB** | ✅ | ✅ | ✅ |
| **FUSE** (sshfs, s3fs, etc.) | ✅ | ✅ | ✅ |
| **OverlayFS** (Docker layers) | ✅ | ✅ | ✅ |
| **UnionFS / AUFS** | ✅ | ✅ | ✅ |
| **memfd / memfile** | ⚠️ Via `/proc/self/exe` | ⚠️ Via `/proc/self/exe` | ⚠️ Via `/proc/self/exe` |
| **execve from memory** | ✅ Via `/proc/self/exe` | ✅ Via `/proc/self/exe` | ✅ Via `/proc/self/exe` |

### Notes on edge cases

- **memfd / memfile**: A process executed from `memfd_create()` still has a readable entry at `/proc/self/exe`. RuntimeShield reads the binary via this path, so binary integrity verification works — provided the memfd content hasn't been modified after exec.
- **execve from memory** (`execve("/proc/self/fd/N")`, `fexecve`, binfmt_misc): Same as memfd — `/proc/self/exe` resolves to the in-memory file and can be read back.
- **tmpfs / ramfs**: Behave like any other filesystem for `read()`. Docker overlay layers commonly use tmpfs. Reads work identically to disk filesystems. No special manifest handling needed — a binary deployed to tmpfs has the same bytes as when it was built.

### Why all filesystems work

RuntimeShield uses `std::fs::read()` which maps to the POSIX `read()` system call. Every filesystem that implements `read()` — which is every filesystem that can store files — supports RuntimeShield. The filesystem type is invisible to the integrity check.

The only exception is `memfd_create()` where a file exists only in memory and has no path that can be re-read. This is a niche case (e.g., in-memory execution of unpacked code).

---

## macOS Version Support

| macOS Version | Process Identity | Anti-Debug | Memory Regions | Library Enumeration |
|---|---|---|---|---|
| **10.15 (Catalina)** | ✅ | ✅ | ✅ | ✅ |
| **11 (Big Sur)** | ✅ | ✅ | ✅ | ✅ |
| **12 (Monterey)** | ✅ | ✅ | ✅ | ✅ |
| **13 (Ventura)** | ✅ | ✅ | ✅ | ✅ |
| **14 (Sonoma)** | ✅ | ✅ | ✅ | ✅ |
| **15 (Sequoia)** | ✅ | ✅ | ✅ | ✅ |

### macOS APIs Used

| Feature | API | Available Since |
|---|---|---|
| Debugger Detection | `sysctl` with `KERN_PROC` + `kinfo_proc.p_flag` | macOS 10.0+ |
| Process Name | `proc_name()` from `libproc.h` | macOS 10.5+ |
| Executable Path | `_NSGetExecutablePath()` | macOS 10.0+ |
| Memory Regions | `mach_vm_region_recurse()` | macOS 10.0+ |
| Memory Read | `mach_vm_read()` | macOS 10.0+ |
| Library Enumeration | `_dyld_get_image_name()` | macOS 10.0+ |

---

## Windows Support (Planned)

Windows support will use equivalent Win32 APIs, not Linux-specific features:

| Feature | Windows API | Status |
|---|---|---|
| Binary Integrity | `ReadFile()` / `CreateFileMapping()` | 🚧 Planned |
| Library Enumeration | `EnumProcessModules()` / `GetModuleFileNameEx()` | 🚧 Planned |
| Memory Regions | `VirtualQueryEx()` | 🚧 Planned |
| Memory Read | `ReadProcessMemory()` | 🚧 Planned |
| Anti-Debug | `IsDebuggerPresent()` / `NtQueryInformationProcess()` | 🚧 Planned |
| Process Identity | `GetCurrentProcessId()` / `GetModuleFileName()` | 🚧 Planned |

All of these are user-mode Win32 APIs available on Windows 10/11, Windows Server 2016+, and do not require kernel drivers or special privileges.

---

## Container & Cloud Support

| Environment | Works? | Notes |
|---|---|---|
| **Docker (Linux containers)** | ✅ Yes | Standard `read()` + `/proc` inside container |
| **Docker (macOS containers)** | ✅ Yes | Runs on macOS host; Linux containers via VM |
| **Kubernetes / podman** | ✅ Yes | Same as Docker |
| **AWS Lambda (custom runtime)** | ✅ Yes | `/proc` available in Lambda execution environment |
| **AWS ECS / EKS** | ✅ Yes | Standard container environment |
| **GitHub Actions CI** | ✅ Yes | Both Linux and macOS runners |
| **GitLab CI** | ✅ Yes | Both Linux and macOS runners |
| **Flatpak / Snap** | ✅ Yes | Can read own binary and /proc |

---

## Summary

| Question | Answer |
|---|---|
| Does it work on Ubuntu 18.04? | ✅ Yes |
| Does it work on Fedora 40 with SELinux? | ✅ Yes |
| Does it work on macOS Sonoma (M3)? | ✅ Yes |
| Does it work on macOS Catalina (Intel)? | ✅ Yes |
| Does it work in a Docker container? | ✅ Yes |
| Does it work on XFS, Btrfs, ZFS? | ✅ Yes |
| Does it work on NFS, FUSE, tmpfs, memfd? | ✅ Yes — reads via file path or `/proc/self/exe` |
| Does it require fs-verity? | ❌ No — pure user-space |
| Does it require root? | ❌ No — runs as the application user |
| Does it require kernel modules? | ❌ No — all user-space APIs |
| Does it work on Windows? | 🚧 Planned (same approach, Win32 APIs) |
