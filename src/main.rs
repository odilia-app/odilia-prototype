#[macro_use]
extern crate lazy_static;
use odilia_common::{
    input::{
        Key,
        KeyBinding,
        Modifiers,
    },
    modes::ScreenReaderMode,
    events::ScreenReaderEventType,
    elements::ElementType,
};
mod keybinds;
mod events;
use crate::events::create_keybind_channel;
use std::{
    sync::Arc,
    sync::Mutex,
    collections::HashMap,
};

use atspi::{enums::AtspiRole, Accessible, Registry};

use futures::stream::StreamExt;
use once_cell::sync::OnceCell;

use tts_subsystem::{Priority, Speaker};

mod state;
use crate::state::{
  ScreenReaderState,
  ScreenReaderEventMap,
};

static STATE: OnceCell<ScreenReaderState<'static>> = OnceCell::new();
static EV_MAP: OnceCell<ScreenReaderEventMap> = OnceCell::new();
static KEY_MAP: OnceCell<Mutex<HashMap<KeyBinding, ScreenReaderEventType>>> = OnceCell::new();

async fn stop_speech(){
    println!("STOP SPEECH");
    STATE
    .get()
    .unwrap()
    .speaker
    .lock()
    .expect("Could not lock speaker.")
    .stop()
    .unwrap();
}
async fn speak(text: impl AsRef<str>) {
    let temp = STATE.get().unwrap().speaker.lock().expect("Unable to lock the speaker.");
    temp.cancel().unwrap();
    temp.speak(Priority::Message, text.as_ref()).unwrap();
}

async fn speak_non_interrupt(text: impl AsRef<str>) {
    STATE.get().unwrap().speaker
        .lock()
        .expect("Unable to lock the speaker")
        .speak(Priority::Important, text.as_ref())
        .unwrap();
}

async fn find_a11y_element(role: AtspiRole, reverse: bool) {
    let focused = STATE.get().unwrap().focus.lock().expect("Cannot lock the STATE.focus mutes");
    if focused.is_some() {
      if let Some(prev_header) = focused.as_ref().expect("This will never happen.").find_role(role, reverse).await.unwrap() {
          prev_header.focus().await.unwrap();
      }
    } else {
      speak("Not found").await;
    }
}

#[inline(always)]
async fn nothing() {
    assert!(true);
}

async fn run_event_func(sret: &ScreenReaderEventType) {
  let map = EV_MAP.get().expect("Could not get EV_MAP");
  let func = map.get(sret).expect("Cannot find screen reader event type requested.");
  func().await;
}

async fn keybind_listener(state: &'static ScreenReaderState<'static>) {
    // this means that a keybinding CANNOT be added later, it must be setup once and used forever.
    let kbdngs: Vec<KeyBinding> = KEY_MAP.get().expect("Cannot get key map").lock().expect("Cannot lock key map").keys().cloned().collect();
    let mut rx = create_keybind_channel(state, &kbdngs);
    while let Some(kb) = rx.recv().await {
        println!("KB: {:?}", kb);
        //tx.send().await;
    }
}

async fn event_listener(state: &'static ScreenReaderState<'static>) {
    let reg = Registry::new()
        .await
        .expect("Unable to register with a11y registry.");
    let mmatch = reg.subscribe_all().await.unwrap();
    let (_mmatch, mut stream) = mmatch.msg_stream();
    while let Some(msg) = stream.next().await {
        let sender = msg.sender().unwrap().into_static();
        let path = msg.path().unwrap().into_static();
        let acc = Accessible::new(
            sender.clone(),
            path.clone(),
            Arc::clone(&reg.proxy.connection),
        );
        if let mut focused_oc = state.focus.lock().expect("Could not lock focus.") {
          *focused_oc = Some(acc.clone());
        }
        let name = acc.get_text().await;
        let role = acc.localized_role_name().await;
        if name.is_ok() && role.is_ok() {
            speak(format!("{}, {}", name.unwrap(), role.unwrap())).await;
        }
    }
}

/// Setup initial state.
async fn init_state() {
    let state = ScreenReaderState {
        mode: Mutex::new(ScreenReaderMode::new("BrowseMode")),
        focus: Mutex::new(None),
        speaker: Mutex::new(Speaker::new("odilia").unwrap()),
        //etf_map: HashMap::new(),
    };
    let _res1 = STATE.set(state);
    let map = HashMap::new();
    let _res2 = EV_MAP.set(map);
    let map2 = Mutex::new(HashMap::new());
    let _res2 = KEY_MAP.set(map2);
}

async fn add_keybind(kbn: KeyBinding, sret: ScreenReaderEventType) {
    let mut map = KEY_MAP.get().expect("Cannot get key map").lock().expect("Could not lock key map");
    map.insert(kbn, sret);
}

#[tokio::main]
async fn main() -> Result<(), dbus::Error> {
    init_state().await;
    //I am trying to fix this by making TTS not be lazily initialised
    speak_non_interrupt("welcome to odilia!").await;
    // always consume caps lock
    let ocap = KeyBinding {
        key: None,
        mods: Modifiers::ODILIA,
        repeat: 1,
        consume: true,
        mode: None,
        notify: false,
    };
//trap the ctrl key, to always stop speech
let stop_speech_key = KeyBinding {
    key: None,
    mods: Modifiers::CONTROL,
    repeat: 1,
    consume: false,
    mode: None,
    notify: true,
};
    
let find_in_tree_kb = KeyBinding {
        key: Some(Key::Other('f')),
        mods: Modifiers::ODILIA,
        repeat: 1,
        consume: true,
        mode: Some(ScreenReaderMode::new("BrowseMode")),
        notify: true,
    };
    //add_keybind(stop_speech_key, ...).await;
    //add_keybind(ocap, nothing).await;
    let next_header_evt = ScreenReaderEventType::Next(ElementType::Heading);
    add_keybind("h".parse().unwrap(), next_header_evt).await;
    add_keybind("Shift+h".parse().unwrap(), ScreenReaderEventType::Previous(ElementType::Heading)).await;
    add_keybind("k".parse().unwrap(), ScreenReaderEventType::Next(ElementType::Link)).await;
    add_keybind("Shift+k".parse().unwrap(), ScreenReaderEventType::Previous(ElementType::Link)).await;
    add_keybind("Odilia+b".parse().unwrap(), ScreenReaderEventType::ChangeMode(ScreenReaderMode::new("BrowseMode"))).await;
    add_keybind("Odilia+a".parse().unwrap(), ScreenReaderEventType::ChangeMode(ScreenReaderMode::new("FocusMode"))).await;
    //add_keybind(find_in_tree_kb, ...).await;

    println!("STARTING ODILIA!");
    let state = STATE.get().unwrap();
    let h1 = tokio::spawn(keybind_listener(state));
    let h2 = tokio::spawn(event_listener(state));
    let _res = tokio::join!(h1, h2);
    Ok(())
}
