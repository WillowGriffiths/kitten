# kitten-rs

A hobby kernel in rust, very early in development at the moment.

## Features

- 64-bit RISC-V support
- Device tree parsing
- Memory allocation

## Plans

- Memory deallocation
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

If you have `cargo`, `rustup` and `qemu-system-riscv64` installed, running the
kernel should be as simple as `cargo run`. No real hardware is supported at this
time. If all goes to plan, you should see some messages printed by the kernel;
that's all it can do at this time.
