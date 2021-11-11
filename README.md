# cobaltOS

[![Rust](https://github.com/CobaltKernel/cobaltOS/actions/workflows/rust.yml/badge.svg)](https://github.com/CobaltKernel/cobaltOS/actions/workflows/rust.yml)

## Overview

- 64-bit x86 Kernel
- BIOS Booting


## Features

- [x] PCI Devices
- [x] Linked List Memory Allocator 
- [ ] Usermode
- [x] System Calls
- [ ] ELF Binary Loading
- [ ] Flat Binary Loading
- [ ] Assembler
- [ ] Text Editor
- [ ] EXT2 Filesystem
- [ ] FAT32 Filesystem
- [x] USTAR Filesystem (Read Only)
- [ ] UEFI Booting
- [x] BIOS Booting
- [x] x86-64
- [ ] ARM
- [ ] RISC-V
- [ ] Bare Bones Shell
- [ ] LibC Implementation
- [ ] Defined ABI


## Build Guide

### Ubuntu (Recommended)

1. Install Rust
```bash
apt-get install build-essential
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```
2. Clone The Repo
```bash
git clone https://github.com/CobaltKernel/cobaltOS.git
```

3. Install prerequirements
```bash
cargo install bootimage
cargo install cargo-release
rustup component add rust-src
rustup component add llvm-tools-preview
```

- Build
```bash
make build
```

- Run
```bash
make run
```
