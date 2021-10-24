use atspi::accessible::AccessibleExt;
use tts_subsystem::Error;
use tts_subsystem::Priority;
use tts_subsystem::Speaker;
fn main() {
    if let Err(e) = atspi::init() {
        eprintln!("Error initialising libatspi: {}", e);
        std::process::exit(1);
    }
    let tts=Speaker::new("yggdrasil").unwrap();
    let desktop = atspi::desktop(0).expect("Desktop 0 should exist");
    //the following bits of code have a lot of unwrap, will refactor this later
    tts.speak(Priority::Message, "this program speaks all visible accessible widgets on the screen, including on windows that might not be visible on the screen or to orca").unwrap();
    for (i, child) in desktop
        .children()
        .expect("Could not get number of children of desktop")
        .enumerate()
    {
        let child = child.expect("Could not get child of desktop");
        let name = child.name().expect("Could not get name");
        let child_count = child.child_count().expect("Could not get child count");
        tts.speak(Priority::Important, &name).unwrap();
    }
}