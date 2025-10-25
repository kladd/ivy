# Ivy

A 64-bit kernel written in hard tabs for the raspberry pi.

## Prerequsites

* rust
* qemu-system-aarch64
* aarch64-linux-gnu-gcc

## Building

#### Once

```bash
cargo install --path cargo-nob
```

#### Compile & Run

```bash
# build
cargo nob

# run (emulator)
cargo nob run
```

