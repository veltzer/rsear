# Architecture

## Module Structure

```
src/
├── main.rs    — entry point, spawns audio thread and launches GUI
├── audio.rs   — audio synthesis and playback using fluidlite + rodio
├── piano.rs   — GUI rendering (grand staff, piano keyboard) using eframe/egui
└── state.rs   — shared state between audio and GUI threads
```

## Threading Model

RSEar uses two threads:

- **Main thread** — runs the eframe/egui GUI event loop
- **Audio thread** — runs `play_sequence()`, synthesizes and plays audio

The threads communicate through `Arc<Mutex<NoteState>>`, which contains the set of currently active MIDI notes and a finished flag.

## Dependencies

| Crate | Purpose |
|---|---|
| `fluidlite` | MIDI synthesizer using SoundFont files |
| `rodio` | Cross-platform audio output |
| `eframe` | GUI framework (wraps egui) |

## Audio Pipeline

1. FluidSynth renders MIDI note data to PCM audio in 20ms chunks (44.1 kHz, stereo)
2. Chunks are queued into a rodio `Sink` for playback
3. The audio thread updates `NoteState` when notes start and stop
4. The GUI thread reads `NoteState` to highlight active notes and continuously repaints while audio is playing
