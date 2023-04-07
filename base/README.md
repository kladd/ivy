# Ivy

A 32-bit kernel written in hard tabs.


## Build

```sh
cargo build

# or

make
```

## Run

([QEMU](https://www.qemu.org/download/) required)

```bash
qemu-system-i386 -kernel target/kernel/ivy -serial stdio

# or

make run
```