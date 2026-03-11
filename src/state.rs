use std::collections::HashSet;

#[derive(Default)]
pub struct NoteState {
    pub active_notes: HashSet<u8>,
    pub finished: bool,
}
