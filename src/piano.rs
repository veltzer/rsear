use std::collections::HashSet;
use std::sync::{Arc, Mutex};

use eframe::egui;

use crate::state::NoteState;

// MIDI note range to display (C3 to C5 = 2 octaves)
const LOW_NOTE: u8 = 48;
const HIGH_NOTE: u8 = 72;

pub struct PianoApp {
    note_state: Arc<Mutex<NoteState>>,
}

impl PianoApp {
    pub fn new(note_state: Arc<Mutex<NoteState>>) -> Self {
        Self { note_state }
    }
}

fn is_black_key(note: u8) -> bool {
    matches!(note % 12, 1 | 3 | 6 | 8 | 10)
}

/// Count white keys in the range [low, high]
fn white_key_count(low: u8, high: u8) -> usize {
    (low..=high).filter(|n| !is_black_key(*n)).count()
}

/// Map a white-key MIDI note to its index among white keys starting from LOW_NOTE
fn white_key_index(note: u8) -> usize {
    (LOW_NOTE..note).filter(|n| !is_black_key(*n)).count()
}

/// Get the white key index to the left of a black key
fn black_key_left_white(note: u8) -> usize {
    // The white key immediately below this black key
    white_key_index(note - 1)
}

/// Convert MIDI note to staff position (semitone-aware, relative to middle C = 0)
/// Returns the number of staff steps from middle C (C4=60).
/// C=0, D=1, E=2, F=3, G=4, A=5, B=6, then +7 per octave.
fn midi_to_staff_position(note: u8) -> i32 {
    let octave = (note as i32 / 12) - 5; // octave relative to C4's octave
    let pitch_class = note % 12;
    let step = match pitch_class {
        0 => 0,        // C
        1 => 0,        // C#
        2 => 1,        // D
        3 => 1,        // D#
        4 => 2,        // E
        5 => 3,        // F
        6 => 3,        // F#
        7 => 4,        // G
        8 => 4,        // G#
        9 => 5,        // A
        10 => 5,       // A#
        11 => 6,       // B
        _ => 0,
    };
    octave * 7 + step
}

fn needs_sharp(note: u8) -> bool {
    matches!(note % 12, 1 | 3 | 6 | 8 | 10)
}

fn draw_staff(ui: &mut egui::Ui, active_notes: &HashSet<u8>) {
    let available_width = ui.available_width();
    let staff_height = 160.0;
    let (response, painter) =
        ui.allocate_painter(egui::vec2(available_width, staff_height), egui::Sense::hover());
    let rect = response.rect;

    let line_color = egui::Color32::from_rgb(60, 60, 60);
    let note_color = egui::Color32::from_rgb(40, 40, 40);
    let sharp_color = egui::Color32::from_rgb(40, 40, 40);

    // Staff parameters
    let line_spacing = 10.0_f32;
    let staff_left = rect.left() + 40.0;
    let staff_right = rect.right() - 20.0;

    // Treble clef: lines are E4, G4, B4, D5, F5 (staff positions 2, 4, 6, 8, 10)
    // Bass clef: lines are G2, B2, D3, F3, A3 (staff positions -7, -5, -3, -1, 1... wait)
    // Let's use a grand staff. Middle C (position 0) sits between the two staves.

    // Center of the grand staff (where middle C ledger line goes)
    let middle_c_y = rect.top() + staff_height * 0.5;

    // Treble staff: bottom line = E4 (pos 2), lines at pos 2,4,6,8,10
    // The bottom line of treble staff is 2 steps above middle C
    // Each step = line_spacing / 2 (since lines are every other step)
    let half_step = line_spacing / 2.0;

    // Y position for a given staff position (higher position = higher on screen = lower y)
    let y_for_pos = |pos: i32| -> f32 { middle_c_y - pos as f32 * half_step };

    // Draw treble clef lines (E4=2, G4=4, B4=6, D5=8, F5=10)
    let treble_lines = [2, 4, 6, 8, 10];
    for &pos in &treble_lines {
        let y = y_for_pos(pos);
        painter.line_segment(
            [egui::pos2(staff_left, y), egui::pos2(staff_right, y)],
            egui::Stroke::new(1.0, line_color),
        );
    }

    // Draw bass clef lines (G2=-7, B2=-5, D3=-3, F3=-1, A3=1... )
    // Actually: G2=midi 43. Let's recalc properly.
    // Bass clef lines: G2, B2, D3, F3, A3
    // G2: octave=(43/12)-5= 3-5=-2, step=4 => -2*7+4=-10
    // B2: midi 47: octave=(47/12)-5=3-5=-2, step=6 => -8
    // D3: midi 50: octave=(50/12)-5=4-5=-1, step=1 => -6
    // F3: midi 53: octave=-1, step=3 => -4
    // A3: midi 57: octave=-1, step=5 => -2
    let bass_lines = [-10, -8, -6, -4, -2];
    for &pos in &bass_lines {
        let y = y_for_pos(pos);
        painter.line_segment(
            [egui::pos2(staff_left, y), egui::pos2(staff_right, y)],
            egui::Stroke::new(1.0, line_color),
        );
    }

    // Draw clef labels
    painter.text(
        egui::pos2(staff_left - 20.0, y_for_pos(6)),
        egui::Align2::CENTER_CENTER,
        "\u{1D11E}", // treble clef unicode (may not render, fallback below)
        egui::FontId::proportional(28.0),
        line_color,
    );
    painter.text(
        egui::pos2(staff_left - 20.0, y_for_pos(-6)),
        egui::Align2::CENTER_CENTER,
        "\u{1D122}", // bass clef unicode
        egui::FontId::proportional(28.0),
        line_color,
    );

    // Draw notes
    let center_x = (staff_left + staff_right) / 2.0;
    let note_radius = half_step * 0.85;
    let mut sorted_notes: Vec<u8> = active_notes.iter().copied().collect();
    sorted_notes.sort();

    // Spread notes horizontally if multiple
    let spread = if sorted_notes.len() > 1 { 30.0 } else { 0.0 };
    let total_spread = spread * (sorted_notes.len() as f32 - 1.0);
    let start_x = center_x - total_spread / 2.0;

    for (i, &note) in sorted_notes.iter().enumerate() {
        let pos = midi_to_staff_position(note);
        let y = y_for_pos(pos);
        let x = start_x + i as f32 * spread;

        // Draw ledger lines if needed
        // Middle C (pos 0) needs a ledger line
        if pos == 0 {
            painter.line_segment(
                [egui::pos2(x - note_radius - 4.0, y), egui::pos2(x + note_radius + 4.0, y)],
                egui::Stroke::new(1.0, line_color),
            );
        }
        // Ledger lines below bass staff (pos < -10)
        {
            let mut lp = -12;
            while lp >= pos {
                let ly = y_for_pos(lp);
                painter.line_segment(
                    [egui::pos2(x - note_radius - 4.0, ly), egui::pos2(x + note_radius + 4.0, ly)],
                    egui::Stroke::new(1.0, line_color),
                );
                lp -= 2;
            }
        }
        // Ledger lines above treble staff (pos > 10)
        {
            let mut lp = 12;
            while lp <= pos {
                let ly = y_for_pos(lp);
                painter.line_segment(
                    [egui::pos2(x - note_radius - 4.0, ly), egui::pos2(x + note_radius + 4.0, ly)],
                    egui::Stroke::new(1.0, line_color),
                );
                lp += 2;
            }
        }

        // Draw the note head (filled ellipse)
        painter.circle_filled(egui::pos2(x, y), note_radius, note_color);

        // Draw sharp symbol if needed
        if needs_sharp(note) {
            painter.text(
                egui::pos2(x - note_radius - 10.0, y),
                egui::Align2::CENTER_CENTER,
                "#",
                egui::FontId::proportional(14.0),
                sharp_color,
            );
        }
    }
}

fn draw_piano(ui: &mut egui::Ui, active_notes: &HashSet<u8>) {
    let available = ui.available_size();
    let (response, painter) =
        ui.allocate_painter(egui::vec2(available.x, available.y), egui::Sense::hover());
    let rect = response.rect;

    let num_white = white_key_count(LOW_NOTE, HIGH_NOTE) as f32;
    let white_w = rect.width() / num_white;
    let white_h = rect.height();
    let black_w = white_w * 0.6;
    let black_h = white_h * 0.62;

    let active_white = egui::Color32::from_rgb(100, 160, 255);
    let active_black = egui::Color32::from_rgb(60, 120, 220);
    let white_color = egui::Color32::WHITE;
    let black_color = egui::Color32::from_rgb(30, 30, 30);
    let outline = egui::Color32::from_rgb(80, 80, 80);

    // Draw white keys
    let mut wi = 0;
    for note in LOW_NOTE..=HIGH_NOTE {
        if is_black_key(note) {
            continue;
        }
        let x = rect.left() + wi as f32 * white_w;
        let key_rect = egui::Rect::from_min_size(egui::pos2(x, rect.top()), egui::vec2(white_w, white_h));
        let fill = if active_notes.contains(&note) {
            active_white
        } else {
            white_color
        };
        painter.rect_filled(key_rect, 2.0, fill);
        painter.rect_stroke(key_rect, 2.0, egui::Stroke::new(1.0, outline), egui::StrokeKind::Outside);

        // Draw note name on the key
        let name = note_name(note);
        painter.text(
            egui::pos2(x + white_w / 2.0, rect.bottom() - 20.0),
            egui::Align2::CENTER_CENTER,
            name,
            egui::FontId::proportional(11.0),
            egui::Color32::from_rgb(120, 120, 120),
        );

        wi += 1;
    }

    // Draw black keys on top
    for note in LOW_NOTE..=HIGH_NOTE {
        if !is_black_key(note) {
            continue;
        }
        let left_white = black_key_left_white(note);
        let x = rect.left() + left_white as f32 * white_w + white_w - black_w / 2.0;
        let key_rect =
            egui::Rect::from_min_size(egui::pos2(x, rect.top()), egui::vec2(black_w, black_h));
        let fill = if active_notes.contains(&note) {
            active_black
        } else {
            black_color
        };
        painter.rect_filled(key_rect, 2.0, fill);
    }
}

fn note_name(note: u8) -> &'static str {
    match note % 12 {
        0 => "C",
        2 => "D",
        4 => "E",
        5 => "F",
        7 => "G",
        9 => "A",
        11 => "B",
        _ => "",
    }
}

impl eframe::App for PianoApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let (active, finished) = {
            let s = self.note_state.lock().unwrap();
            (s.active_notes.clone(), s.finished)
        };

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Piano Visualizer");
            ui.add_space(8.0);

            if finished {
                ui.label("Playback finished.");
            } else {
                let names: Vec<&str> = {
                    let mut notes: Vec<u8> = active.iter().copied().collect();
                    notes.sort();
                    notes.iter().map(|&n| midi_to_full_name(n)).collect()
                };
                if names.is_empty() {
                    ui.label("...");
                } else {
                    ui.label(format!("Playing: {}", names.join(" ")));
                }
            }

            ui.add_space(8.0);
            draw_staff(ui, &active);
            ui.add_space(8.0);
            draw_piano(ui, &active);
        });

        // Continuously repaint while audio is playing
        if !finished {
            ctx.request_repaint();
        }
    }
}

fn midi_to_full_name(note: u8) -> &'static str {
    match note {
        48 => "C3",
        49 => "C#3",
        50 => "D3",
        51 => "D#3",
        52 => "E3",
        53 => "F3",
        54 => "F#3",
        55 => "G3",
        56 => "G#3",
        57 => "A3",
        58 => "A#3",
        59 => "B3",
        60 => "C4",
        61 => "C#4",
        62 => "D4",
        63 => "D#4",
        64 => "E4",
        65 => "F4",
        66 => "F#4",
        67 => "G4",
        68 => "G#4",
        69 => "A4",
        70 => "A#4",
        71 => "B4",
        72 => "C5",
        _ => "?",
    }
}
