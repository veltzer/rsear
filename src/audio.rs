use std::sync::{Arc, Mutex};

use fluidlite::{Settings, Synth};
use rodio::buffer::SamplesBuffer;
use rodio::{OutputStream, Sink};

use crate::state::{NoteState, lock};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

const SOUNDFONT_PATH: &str = "/usr/share/sounds/sf2/FluidR3_GM.sf2";
const SAMPLE_RATE: u32 = 44100;
const CHANNELS: u16 = 2;
const CHUNK_MS: usize = 20;

const fn chunk_samples() -> usize {
    SAMPLE_RATE as usize * CHANNELS as usize * CHUNK_MS / 1000
}

fn render_chunks(synth: &Synth, sink: &Sink, count: usize) -> Result<()> {
    let size = chunk_samples();
    for _ in 0..count {
        let mut buf = vec![0f32; size];
        synth.write(buf.as_mut_slice())?;
        sink.append(SamplesBuffer::new(CHANNELS, SAMPLE_RATE, buf));
        // Pace rendering with playback so note state changes stay in sync with audio
        // Keep a larger buffer to avoid underruns/hiccups
        while sink.len() > 10 {
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
    }
    Ok(())
}

fn play_note(
    synth: &Synth,
    sink: &Sink,
    note: u8,
    velocity: u32,
    duration_ms: usize,
    state: &Arc<Mutex<NoteState>>,
) -> Result<()> {
    synth.note_on(0, u32::from(note), velocity)?;
    lock(state).active_notes.insert(note);

    let chunks = duration_ms / CHUNK_MS;
    render_chunks(synth, sink, chunks)?;

    synth.note_off(0, u32::from(note))?;
    lock(state).active_notes.remove(&note);

    // Short release tail
    render_chunks(synth, sink, 3)
}

/// Play the demo: a C major scale, then the chord. Fails when the synth, the
/// soundfont or the audio device is not available.
pub fn play_sequence(state: &Arc<Mutex<NoteState>>) -> Result<()> {
    let settings = Settings::new()?;
    let synth = Synth::new(settings)?;
    synth.sfload(SOUNDFONT_PATH, true)?;
    synth.set_gain(2.0);

    let (_stream, stream_handle) = OutputStream::try_default()?;
    let sink = Sink::try_new(&stream_handle)?;

    // C major scale
    let notes: &[u8] = &[60, 62, 64, 65, 67, 69, 71, 72];
    for &note in notes {
        play_note(&synth, &sink, note, 100, 400, state)?;
    }

    // Pause between scale and chord
    render_chunks(&synth, &sink, 10)?;

    // C major chord
    let chord: &[u8] = &[60, 64, 67];
    for &note in chord {
        synth.note_on(0, u32::from(note), 100)?;
    }
    {
        let mut s = lock(state);
        for &note in chord {
            s.active_notes.insert(note);
        }
    }

    render_chunks(&synth, &sink, 2000 / CHUNK_MS)?;

    for &note in chord {
        synth.note_off(0, u32::from(note))?;
    }
    {
        let mut s = lock(state);
        for &note in chord {
            s.active_notes.remove(&note);
        }
    }

    // Release tail
    render_chunks(&synth, &sink, 50)?;

    sink.sleep_until_end();
    lock(state).finished = true;
    Ok(())
}
