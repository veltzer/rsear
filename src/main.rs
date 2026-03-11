use std::thread;
use std::time::Duration;

use fluidlite::{Settings, Synth};
use rodio::buffer::SamplesBuffer;
use rodio::{OutputStream, Sink};

const SOUNDFONT_PATH: &str = "/usr/share/sounds/sf2/FluidR3_GM.sf2";
const SAMPLE_RATE: u32 = 44100;
const CHANNELS: u16 = 2;

fn main() {
    let settings = Settings::new().unwrap();
    let synth = Synth::new(settings).unwrap();
    synth.sfload(SOUNDFONT_PATH, true).unwrap();
    synth.set_gain(2.0);

    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();

    // Play a C major scale: C4 D4 E4 F4 G4 A4 B4 C5
    let notes = [60, 62, 64, 65, 67, 69, 71, 72];
    let note_duration_ms = 400;
    let note_samples = SAMPLE_RATE as usize * CHANNELS as usize * note_duration_ms / 1000;

    for &note in &notes {
        synth.note_on(0, note, 100).unwrap();

        let mut buffer = vec![0f32; note_samples];
        synth.write(buffer.as_mut_slice()).unwrap();

        synth.note_off(0, note).unwrap();

        // Render a short tail for the note release
        let tail_samples = SAMPLE_RATE as usize * CHANNELS as usize * 50 / 1000;
        let mut tail = vec![0f32; tail_samples];
        synth.write(tail.as_mut_slice()).unwrap();

        buffer.extend_from_slice(&tail);
        let source = SamplesBuffer::new(CHANNELS, SAMPLE_RATE, buffer);
        sink.append(source);
    }

    // Play a final C major chord
    thread::sleep(Duration::from_millis(200));
    synth.note_on(0, 60, 100).unwrap(); // C4
    synth.note_on(0, 64, 100).unwrap(); // E4
    synth.note_on(0, 67, 100).unwrap(); // G4

    let chord_samples = SAMPLE_RATE as usize * CHANNELS as usize * 2; // 2 seconds
    let mut chord_buf = vec![0f32; chord_samples];
    synth.write(chord_buf.as_mut_slice()).unwrap();

    synth.note_off(0, 60).unwrap();
    synth.note_off(0, 64).unwrap();
    synth.note_off(0, 67).unwrap();

    // Release tail for chord
    let mut chord_tail = vec![0f32; SAMPLE_RATE as usize * CHANNELS as usize];
    synth.write(chord_tail.as_mut_slice()).unwrap();
    chord_buf.extend_from_slice(&chord_tail);

    let source = SamplesBuffer::new(CHANNELS, SAMPLE_RATE, chord_buf);
    sink.append(source);

    sink.sleep_until_end();
}
