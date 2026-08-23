#!/bin/bash
# Install the system libraries this repo's build links against and the
# external tools its tests shell out to. The canonical ci.yml runs this in
# every repo before `cargo build`; a repo that needs neither keeps this
# script as an explicit no-op. Keep it strict: anything that fails to
# install must fail the build here, not surface later as a confusing
# build or test failure.
set -euo pipefail

# rsear plays audio through cpal, whose alsa-sys build script probes
# pkg-config for the ALSA headers — without libasound2-dev the build fails
# before compiling a single Rust file.
#
# Acquire::Retries because apt's default is 0: when the first mirror in
# /etc/apt/apt-mirrors.txt is unreachable there is no second attempt;
# Retries=3 lets apt fall through to archive.ubuntu.com.
sudo apt-get -o Acquire::Retries=3 update
sudo apt-get -o Acquire::Retries=3 install -y libasound2-dev

# The tests synthesize audio through fluidsynth against the FluidR3 General
# MIDI soundfont at its packaged path (/usr/share/sounds/sf2/FluidR3_GM.sf2,
# the SOUNDFONT_PATH constant in src/audio.rs and tests/basic.rs).
sudo apt-get -o Acquire::Retries=3 install -y fluid-soundfont-gm
