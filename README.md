# kitten-rs

A hobby kernel in rust, very early in development at the moment.

## Features

- 64-bit RISC-V support
- Device tree parsing
- Custom memory allocator (buddy and slab)

## Plans

- Custom filesystem
- Run on the VisionFive 2
- Full SMT support
- Working network stack

Ultimately, kitten is a project focused on developing my skills and knowledge
with bare metal RISC-V (and perhaps other arches down the road). Not every
feature I implement will be a good idea, because that's part of the fun! Some
non-goals include:

- POSIX compliance
- ABI stability
- Software compatibility

## Running

### Dependencies

Aside from rust itself, the only dependency is `qemu-system-riscv64`:

```sh
# On Debian/Ubuntu
$ sudo apt install qemu-system-misc

# On Arch
$ sudo pacman -S qemu-system-riscv

# On Fedora
$ sudo dnf install qemu-system-riscv

# On MacOS
$ brew install qemu
```

Make sure you install rust through rustup so that cargo can install the RISC-V
target for you automatically.

### Building and running

```
cargo run
```
