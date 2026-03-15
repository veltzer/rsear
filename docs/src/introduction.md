# RSEar - Train Your Ears

A GUI-based ear training application written in Rust that helps users develop their musical ear through real-time visual feedback.

## Features

- **Real-time note visualization** — see notes as they are played on a grand staff and piano keyboard
- **Grand staff display** — treble and bass clef notation with proper ledger lines
- **Piano keyboard** — interactive two-octave keyboard (C3–C5) highlighting active notes
- **Audio synthesis** — MIDI-based sound generation using FluidSynth and SoundFont files
- **Concurrent playback** — audio runs on a separate thread with synchronized GUI updates

## How It Works

RSEar plays a musical sequence (C major scale followed by a C major chord) while displaying the notes in three simultaneous views:

1. Musical staff notation (grand staff with treble and bass clefs)
2. Piano keyboard with highlighted keys
3. Text display of note names (e.g., "C4", "G4")
