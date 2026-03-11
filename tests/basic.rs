use fluidlite::{Settings, Synth};
use rodio::buffer::SamplesBuffer;
use rodio::Source;

const SOUNDFONT_PATH: &str = "/usr/share/sounds/sf2/FluidR3_GM.sf2";
const SAMPLE_RATE: u32 = 44100;
const CHANNELS: u16 = 2;

fn create_synth() -> Synth {
    let settings = Settings::new().unwrap();
    let synth = Synth::new(settings).unwrap();
    synth.sfload(SOUNDFONT_PATH, true).unwrap();
    synth
}

#[test]
fn test_synth_creation() {
    let settings = Settings::new().unwrap();
    let synth = Synth::new(settings).unwrap();
    assert!(synth.sfload(SOUNDFONT_PATH, true).is_ok());
}

#[test]
fn test_note_on_off() {
    let synth = create_synth();
    assert!(synth.note_on(0, 60, 100).is_ok());
    assert!(synth.note_off(0, 60).is_ok());
}

#[test]
fn test_note_produces_audio() {
    let synth = create_synth();
    synth.note_on(0, 60, 100).unwrap();

    let num_samples = SAMPLE_RATE as usize * CHANNELS as usize / 10; // 100ms
    let mut buffer = vec![0f32; num_samples];
    synth.write(buffer.as_mut_slice()).unwrap();

    // After triggering a note, the buffer should contain non-zero samples
    let has_audio = buffer.iter().any(|&s| s != 0.0);
    assert!(has_audio, "Expected non-zero audio samples after note_on");
}

#[test]
fn test_silence_without_note() {
    let synth = create_synth();

    let num_samples = SAMPLE_RATE as usize * CHANNELS as usize / 10;
    let mut buffer = vec![0f32; num_samples];
    synth.write(buffer.as_mut_slice()).unwrap();

    // Without any note playing, the output should be near-silent
    let peak = buffer.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
    assert!(
        peak < 0.01,
        "Expected near-silence when no note is playing, but peak was {peak}"
    );
}

#[test]
fn test_different_notes_produce_different_audio() {
    let synth = create_synth();

    // Render middle C
    synth.note_on(0, 60, 100).unwrap();
    let num_samples = SAMPLE_RATE as usize * CHANNELS as usize / 10;
    let mut buf_c = vec![0f32; num_samples];
    synth.write(buf_c.as_mut_slice()).unwrap();
    synth.note_off(0, 60).unwrap();

    // Drain residual audio
    let mut drain = vec![0f32; SAMPLE_RATE as usize * CHANNELS as usize];
    synth.write(drain.as_mut_slice()).unwrap();

    // Render A4
    synth.note_on(0, 69, 100).unwrap();
    let mut buf_a = vec![0f32; num_samples];
    synth.write(buf_a.as_mut_slice()).unwrap();
    synth.note_off(0, 69).unwrap();

    assert_ne!(buf_c, buf_a, "Different notes should produce different audio");
}

#[test]
fn test_velocity_affects_amplitude() {
    let synth = create_synth();

    // Soft note
    synth.note_on(0, 60, 30).unwrap();
    let num_samples = SAMPLE_RATE as usize * CHANNELS as usize / 10;
    let mut buf_soft = vec![0f32; num_samples];
    synth.write(buf_soft.as_mut_slice()).unwrap();
    synth.note_off(0, 60).unwrap();

    let mut drain = vec![0f32; SAMPLE_RATE as usize * CHANNELS as usize];
    synth.write(drain.as_mut_slice()).unwrap();

    // Loud note
    synth.note_on(0, 60, 127).unwrap();
    let mut buf_loud = vec![0f32; num_samples];
    synth.write(buf_loud.as_mut_slice()).unwrap();
    synth.note_off(0, 60).unwrap();

    let peak_soft = buf_soft.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
    let peak_loud = buf_loud.iter().map(|s| s.abs()).fold(0.0f32, f32::max);

    assert!(
        peak_loud > peak_soft,
        "Loud note (peak {peak_loud}) should have greater amplitude than soft note (peak {peak_soft})"
    );
}

#[test]
fn test_chord_produces_audio() {
    let synth = create_synth();

    // C major chord
    synth.note_on(0, 60, 100).unwrap();
    synth.note_on(0, 64, 100).unwrap();
    synth.note_on(0, 67, 100).unwrap();

    let num_samples = SAMPLE_RATE as usize * CHANNELS as usize / 10;
    let mut buffer = vec![0f32; num_samples];
    synth.write(buffer.as_mut_slice()).unwrap();

    let has_audio = buffer.iter().any(|&s| s != 0.0);
    assert!(has_audio, "Chord should produce audio");
}

#[test]
fn test_samples_buffer_properties() {
    let synth = create_synth();
    synth.note_on(0, 60, 100).unwrap();

    let num_samples = SAMPLE_RATE as usize * CHANNELS as usize / 10;
    let mut buffer = vec![0f32; num_samples];
    synth.write(buffer.as_mut_slice()).unwrap();

    let source = SamplesBuffer::new(CHANNELS, SAMPLE_RATE, buffer);
    assert_eq!(source.channels(), CHANNELS);
    assert_eq!(source.sample_rate(), SAMPLE_RATE);
}
