# Installation

## Prerequisites

- Rust toolchain (edition 2024)
- FluidR3 GM SoundFont file at `/usr/share/sounds/sf2/FluidR3_GM.sf2`

On Debian/Ubuntu, install the SoundFont:

```bash
sudo apt install fluid-soundfont-gm
```

## Install from crates.io

```bash
cargo install rsear
```

This downloads, compiles, and installs the latest published version into `~/.cargo/bin/`.

## Build from source

```bash
git clone https://github.com/veltzer/rsear.git
cd rsear
cargo build --release
```

The binary will be at `target/release/rsear`.
