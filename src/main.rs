use std::sync::Mutex;
use atspi::accessible::AccessibleExt;
use tts_subsystem::Priority;
use tts_subsystem::Speaker;
use once_cell::sync::OnceCell;
fn speak(text:&str) {
    tts
        .get()
        .unwrap()
        .lock()
        .unwrap()
        .speak(Priority::Important, text)
        .unwrap();
}
static tts: OnceCell<Mutex<Speaker>>=OnceCell::new();
fn main() {
    if let Err(e) = atspi::init() {
        eprintln!("Error initialising libatspi: {}", e);
        std::process::exit(1);
    }
    tts.set(Mutex::new(Speaker::new("yggdrasil").unwrap()));
    let desktop = atspi::desktop(0).expect("Desktop 0 should exist");
    //the following bits of code have a lot of unwrap, will refactor this later
    speak("this program speaks all visible accessible widgets in the accessibility tree, including on windows that might not be visible on the screen or to orca");
    for (i, child) in desktop
        .children()
        .expect("Could not get number of children of desktop")
        .enumerate()
    {
        let child = child.expect("Could not get child of desktop");
        let name = child.name().expect("Could not get name");
        let child_count = child.child_count().expect("Could not get child count");
        speak(&format!("item number: {}", i));
        speak(&name);
        speak(&format!("node number: {}", child_count));
    }
}
