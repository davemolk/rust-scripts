use crossterm::event::{poll, read, KeyEvent, Event};
use std::time::Duration;


pub fn poll_for_event() -> Option<KeyEvent> {
    if poll(Duration::from_millis(100)).ok()? {
        let event = read().ok()?;
        if let Event::Key(key_event) = event {
            return Some(key_event);
        }
    }
    None
}

