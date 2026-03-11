use std::sync::{Arc, Mutex};

use fluidlite::{Settings, Synth};
use rodio::buffer::SamplesBuffer;
use rodio::{OutputStream, Sink};

use crate::state::NoteState;

const SOUNDFONT_PATH: &str = "/usr/share/sounds/sf2/FluidR3_GM.sf2";
const SAMPLE_RATE: u32 = 44100;
const CHANNELS: u16 = 2;
const CHUNK_MS: usize = 20;

fn chunk_samples() -> usize {
    SAMPLE_RATE as usize * CHANNELS as usize * CHUNK_MS / 1000
}

fn render_chunks(synth: &Synth, sink: &Sink, count: usize) {
    let size = chunk_samples();
    for _ in 0..count {
        let mut buf = vec![0f32; size];
        synth.write(buf.as_mut_slice()).unwrap();
        sink.append(SamplesBuffer::new(CHANNELS, SAMPLE_RATE, buf));
    }
}

fn play_note(
    synth: &Synth,
    sink: &Sink,
    note: u8,
    velocity: u32,
    duration_ms: usize,
    state: &Arc<Mutex<NoteState>>,
) {
    synth.note_on(0, note as u32, velocity).unwrap();
    state.lock().unwrap().active_notes.insert(note);

    let chunks = duration_ms / CHUNK_MS;
    render_chunks(synth, sink, chunks);

    synth.note_off(0, note as u32).unwrap();
    state.lock().unwrap().active_notes.remove(&note);

    // Short release tail
    render_chunks(synth, sink, 3);
}

pub fn play_sequence(state: Arc<Mutex<NoteState>>) {
    let settings = Settings::new().unwrap();
    let synth = Synth::new(settings).unwrap();
    synth.sfload(SOUNDFONT_PATH, true).unwrap();
    synth.set_gain(2.0);

    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();

    // C major scale
    let notes: &[u8] = &[60, 62, 64, 65, 67, 69, 71, 72];
    for &note in notes {
        play_note(&synth, &sink, note, 100, 400, &state);
    }

    // Pause between scale and chord
    render_chunks(&synth, &sink, 10);

    // C major chord
    let chord: &[u8] = &[60, 64, 67];
    for &note in chord {
        synth.note_on(0, note as u32, 100).unwrap();
    }
    {
        let mut s = state.lock().unwrap();
        for &note in chord {
            s.active_notes.insert(note);
        }
    }

    render_chunks(&synth, &sink, 2000 / CHUNK_MS);

    for &note in chord {
        synth.note_off(0, note as u32).unwrap();
    }
    {
        let mut s = state.lock().unwrap();
        for &note in chord {
            s.active_notes.remove(&note);
        }
    }

    // Release tail
    render_chunks(&synth, &sink, 50);

    sink.sleep_until_end();
    state.lock().unwrap().finished = true;
}
