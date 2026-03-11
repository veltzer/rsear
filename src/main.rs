mod audio;
mod piano;
mod state;

use std::sync::{Arc, Mutex};

fn main() -> eframe::Result<()> {
    let note_state = Arc::new(Mutex::new(state::NoteState::default()));
    let audio_state = note_state.clone();

    std::thread::spawn(move || {
        // Small delay so the GUI window appears first
        std::thread::sleep(std::time::Duration::from_millis(500));
        audio::play_sequence(audio_state);
    });

    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default().with_inner_size([900.0, 550.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Piano Visualizer",
        options,
        Box::new(|_cc| Ok(Box::new(piano::PianoApp::new(note_state)))),
    )
}
