#[macro_use]
extern crate lazy_static;
use odilia_common::{
    input::{
        Key,
        KeyBinding,
        Modifiers,
    },
    modes::ScreenReaderMode,
};
use odilia_input::{
    events::create_keybind_channel,
    keybinds::{
        add_keybind,
        run_keybind_func,
        set_sr_mode,
    },
};
use std::{
    sync::Arc,
    sync::Mutex as SyncMutex,
};

use atspi::{enums::AtspiRole, Accessible, Registry};

use futures::stream::StreamExt;
use once_cell::sync::OnceCell;
use tokio::sync::Mutex;

use tts_subsystem::{Priority, Speaker};

pub mod state;
use crate::state::ScreenReaderState;

static STATE: OnceCell<ScreenReaderState<'static>> = OnceCell::new();

async fn stop_speech(){
    STATE
    .get()
    .unwrap()
    .speaker
    .lock()
    .await
    .stop()
    .unwrap();
}
async fn speak(text: impl AsRef<str>) {
    let mut temp = STATE.get().unwrap().speaker.lock().await;
    temp.cancel().unwrap();
    temp.speak(Priority::Message, text.as_ref()).unwrap();
}

async fn speak_non_interrupt(text: impl AsRef<str>) {
    STATE.get().unwrap().speaker
        .lock()
        .await
        .speak(Priority::Important, text.as_ref())
        .unwrap();
}

async fn next_link() {
    let focused = FOCUSED_A11Y.get().unwrap().lock().await;
    if let Some(next_header) = focused.find_role(AtspiRole::Link, false).await.unwrap() {
        next_header.focus().await.unwrap();
    }
}
async fn prev_link() {
    let focused = FOCUSED_A11Y.get().unwrap().lock().await;
    if let Some(next_header) = focused.find_role(AtspiRole::Link, true).await.unwrap() {
        next_header.focus().await.unwrap();
    }
}
async fn next_header() {
    let focused = FOCUSED_A11Y.get().unwrap().lock().await;
    if let Some(next_header) = focused.find_role(AtspiRole::Heading, false).await.unwrap() {
        next_header.focus().await.unwrap();
    }
}

async fn find_in_tree() {
    speak("Find in tree").await;
}

async fn previous_header() {
    let focused = FOCUSED_A11Y.get().unwrap().lock().await;
    if let Some(prev_header) = focused.find_role(AtspiRole::Heading, true).await.unwrap() {
        prev_header.focus().await.unwrap();
    }
}

async fn activate_focus_mode() {
    let fm = ScreenReaderMode::new("FocusMode");
    set_sr_mode(fm).await;
    speak("Focus mode").await;
}

async fn activate_browse_mode() {
    let bm = ScreenReaderMode::new("BrowseMode");
    set_sr_mode(bm).await;
    speak("Browse Mode").await;
}
#[inline(always)]
async fn nothing() {
    assert!(true);
}

async fn keybind_listener(state: &ScreenReaderState<'static>) {
    let mut rx = create_keybind_channel();
    println!("WAITING FOR KEYS!");
    while let Some(kb) = rx.recv().await {
        println!("KEY PRESSED");
        // need to do this explicitly for now
        run_keybind_func(&kb).await;
    }
}

async fn event_listener(state: &ScreenReaderState<'static>) {
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
        if let mut focused_oc = state.focus.lock().await {
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
    };
    let _res1 = STATE.set(state);
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
    add_keybind(stop_speech_key, stop_speech).await;
    add_keybind(ocap, nothing).await;
    add_keybind("h".parse().unwrap(), next_header).await;
    add_keybind(find_in_tree_kb, find_in_tree).await;
    add_keybind("Shift+h".parse().unwrap(), previous_header).await;
    add_keybind("k".parse().unwrap(), next_link).await;
    add_keybind("Shift+k".parse().unwrap(), prev_link).await;
    add_keybind("Odilia+b".parse().unwrap(), activate_browse_mode).await;
    add_keybind("Odilia+a".parse().unwrap(), activate_focus_mode).await;

    println!("STARTING ODILIA!");

    let state = STATE.get().unwrap();
    let h1 = tokio::spawn(keybind_listener(state));
    let h2 = tokio::spawn(event_listener(state));
    let _res = tokio::join!(h1, h2);
    Ok(())
}
