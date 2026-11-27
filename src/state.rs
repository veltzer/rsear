use std::collections::HashSet;
use std::sync::{Mutex, MutexGuard, PoisonError};

#[derive(Default)]
pub struct NoteState {
    pub active_notes: HashSet<u8>,
    pub finished: bool,
}

/// Lock the shared note state. A thread that panicked while holding the lock
/// leaves it poisoned; the set of sounding notes is still worth showing, so
/// the guard is taken anyway.
pub fn lock(state: &Mutex<NoteState>) -> MutexGuard<'_, NoteState> {
    state.lock().unwrap_or_else(PoisonError::into_inner)
}
