# TOFIX

Findings from a code scan on 2026-10-04.

## High

- `src/audio.rs:9` - the SoundFont path is hardcoded to the Debian location `/usr/share/sounds/sf2/FluidR3_GM.sf2` and loaded with `.unwrap()` at `src/audio.rs:56`, yet release binaries are built for macOS too (`Cargo.toml:15-25`); on any system without that exact file the audio thread panics. Make the path configurable (CLI flag / env var), search a few known locations, and report a clear error instead of panicking.

## Medium

- `src/main.rs:11-15` - if the audio thread panics (missing SoundFont, no audio device at `src/audio.rs:59`, any of the `.unwrap()`s in `src/audio.rs`), `NoteState::finished` is never set, so the window shows "..." forever and `src/piano.rs:340-342` keeps requesting repaints in a busy loop with no error shown. Have `play_sequence` return a `Result`, record the error in `NoteState`, and display it in the GUI.
- `docs/src/getting-started.md:21` - documents the scale as C3, D3 ... C4, but `src/audio.rs:63` plays MIDI 60..72, which `src/piano.rs:360-372` itself names C4..C5 (and line 23 of the same doc calls 60 "C4"); fix the doc to C4 ... C5.
- `README.md:3` / `docs/src/introduction.md:1-3` - present RSEar as an ear-training program, but the code only plays one fixed C-major scale and chord (`src/audio.rs:62-96`) with no exercise, prompt or user input; describe it as what it is today (a playback visualizer) until training features exist.

## Low

- `docs/src/introduction.md:9` - calls the keyboard "interactive", but `src/piano.rs:226` allocates it with `egui::Sense::hover()` and nothing handles clicks; drop the word or add click-to-play.
- `src/main.rs:22` / `src/piano.rs:315` - window title and heading are "Piano Visualizer", a leftover name; use "RSEar".
- `tests/basic.rs:5` / `src/audio.rs:9` - `SOUNDFONT_PATH`, `SAMPLE_RATE` and `CHANNELS` are duplicated between the binary and the tests; once the path becomes configurable, share it (e.g. move the constants into a small lib target) so the two cannot drift.
