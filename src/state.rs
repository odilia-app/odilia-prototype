use odilia_common::{
  modes::ScreenReaderMode,
  events::ScreenReaderEventType,
};
use std::sync::Mutex;
use tts_subsystem::Speaker;
use atspi::accessible::Accessible;
use std::{
    collections::HashMap,
    future::Future,
};


/// ScreenReaderState stores all information related to global state in the screen reader.
pub struct ScreenReaderState<'a> {
    /// mode: a mode (String) describing the mode of the screenreader
    pub mode: Mutex<ScreenReaderMode>,
    /// focus: the currently focused accessible element as a dbus Proxy
    pub focus: Mutex<Option<Accessible<'a>>>,
    /// speaker: a speaker mutex which can be unlocked by any required green (tokio) threads
    pub speaker: Mutex<Speaker>,
}

pub type AsyncFn = Box<dyn Fn() -> Box<dyn Future<Output=()> + Unpin + Send + 'static> + Send + Sync + 'static>;
pub type ScreenReaderEventMap = HashMap<ScreenReaderEventType, AsyncFn>;
