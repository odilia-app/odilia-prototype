use odilia_common::modes::ScreenReaderMode;
use dbus::nonblock::{
  Proxy,
  SyncConnection,
};
use tokio::sync::Mutex;
use tts_subsystem::Speaker;
use std::sync::Arc;
use atspi::accessible::Accessible;

/// ScreenReaderState stores all information related to global state in the screen reader.
pub struct ScreenReaderState<'a> {
    /// mode: a mode (String) describing the mode of the screenreader
    pub mode: Mutex<ScreenReaderMode>,
    /// focus: the currently focused accessible element as a dbus Proxy
    pub focus: Mutex<Option<Accessible<'a>>>,
    /// speaker: a speaker mutex which can be unlocked by any required green (tokio) threads
    pub speaker: Mutex<Speaker>,
}
