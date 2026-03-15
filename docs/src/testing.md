# Testing

## Running Tests

```bash
cargo test
```

Or with the parallel test runner:

```bash
cargo nextest run
```

## Test Coverage

The integration tests in `tests/basic.rs` cover the audio synthesis layer:

| Test | What it verifies |
|---|---|
| `test_synth_creation` | FluidSynth initializes and SoundFont loads |
| `test_note_on_off` | MIDI note on/off commands work |
| `test_note_produces_audio` | A played note generates non-zero audio samples |
| `test_silence_without_note` | Synth outputs near-silence when no notes play |
| `test_different_notes_produce_different_audio` | Different MIDI notes produce different waveforms |
| `test_velocity_affects_amplitude` | Higher velocity produces louder audio |
| `test_chord_produces_audio` | Simultaneous notes (chord) generate audio |
| `test_samples_buffer_properties` | `SamplesBuffer` has correct channel/sample-rate properties |

All tests require the system SoundFont file to be installed.
