use odilia_common::{
  events::ScreenReaderEventType,
  input::{
    KeyBinding,
    KeyEvent,
    Modifiers,
  },
  modes::{
    ScreenReaderMode,
  },
};
use tokio::{
  sync::Mutex,
};
use std::{
  future::Future,
  collections::HashMap,
};

use crate::state::ScreenReaderState;

lazy_static! {
  static ref KB_MAP: Mutex<HashMap<KeyBinding, ScreenReaderEventType>> = Mutex::new(HashMap::new());
}

pub async fn keyevent_match(kbm: &KeyEvent, state: &'static ScreenReaderState<'static>) -> Option<KeyBinding>
{
  let kbhm = KB_MAP.lock().await;
  let sr_mode = state.mode.lock().expect("Could not lock mode.");
  for (kb, _) in kbhm.iter() {
    let mut matches = true;
    matches &= kb.key == kbm.key;
    matches &= kb.repeat == kbm.repeat;
    matches &= (kb.mods == Modifiers::NONE && kbm.mods == Modifiers::NONE) || kb.mods.intersects(kbm.mods);
    if let Some(mode) = &kb.mode {
      matches &= *mode == *sr_mode;
    }
    if matches {
      return Some(kb.clone());
    }
  }
  None
}

/* this will match with the bitflags */
pub fn keyevent_match_sync(kbm: &KeyEvent, state: &'static ScreenReaderState<'static>, kbs: &Vec<KeyBinding>) -> Option<KeyBinding>
{
  let sr_mode = state.mode.lock().expect("Could not lock mode");
  for kb in kbs {
    let mut matches = true;
    matches &= kb.key == kbm.key;
    matches &= kb.repeat == kbm.repeat;
    matches &= (kb.mods == Modifiers::NONE && kbm.mods == Modifiers::NONE) || kb.mods.intersects(kbm.mods);
    if let Some(mode) = &kb.mode {
      matches &= *mode == *sr_mode;
    }
    if matches {
      return Some(kb.clone());
    }
  }
  None
} 
