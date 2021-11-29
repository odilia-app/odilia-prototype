#[macro_use]
extern crate lazy_static;
use odilia_input::{
  events::create_keybind_channel,
  keybinds::{
    add_keybind,
    run_keybind_func,
    get_sr_mode,
    set_sr_mode,
  },
};
use odilia_common::{
  input::{KeyBinding,KeyEvent,Modifiers,Key},
  modes::ScreenReaderMode,
};
use std::{sync::Arc, sync::Mutex as SyncMutex, time::Duration, collections::HashMap, future::Future};
use rdev::{Event as RDevEvent, EventType};

use atspi::Accessible;

use dbus::{
    channel::Channel,
    message::{MatchRule, SignalArgs},
    nonblock::{stdintf::org_freedesktop_dbus::Properties, MethodReply, Proxy, SyncConnection},
};
use futures::stream::StreamExt;
use once_cell::sync::OnceCell;
use tokio::sync::{Mutex};

use atspi_codegen::event::OrgA11yAtspiEventObjectStateChanged as StateChanged;
use atspi_codegen::event::OrgA11yAtspiEventObjectTextCaretMoved as CaretMoved;
use atspi_codegen::device_event_controller::OrgA11yAtspiDeviceEventController;
use tts_subsystem::{Priority, Speaker};

const TIMEOUT: Duration = Duration::from_secs(1);

//initialise a global tts object
static TTS: OnceCell<Mutex<Speaker>> = OnceCell::new();
lazy_static! {
// TODO: static set init
  static ref ACTIVE_MODE: Arc<SyncMutex<ScreenReaderMode>> = Arc::new(SyncMutex::new(ScreenReaderMode::new("CommandMode")));
}

async fn speak(text: impl AsRef<str>) {
    let temp = TTS.get().unwrap().lock().await;
    temp.cancel().unwrap();
    temp.speak(Priority::Message, text.as_ref()).unwrap();
}

async fn speak_non_interrupt(text: impl AsRef<str>) {
    TTS.get().unwrap().lock().await.speak(Priority::Important, text.as_ref()).unwrap();
}

async fn next_header() {
  speak("Next header").await;
}

async fn find_in_tree() {
  speak("Find in tree").await;
}

async fn previous_header() {
  speak("Previous header").await;
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

async fn nothing(){}

#[tokio::main]
async fn main() -> Result<(), dbus::Error> {
    // always consume caps lock
    let ocap = KeyBinding {
        key: None,
        mods: Modifiers::ODILIA,
        repeat: 1,
        consume: true,
        mode: None,
        notify: false,
    };
    let find_in_tree_kb = KeyBinding {
        key: Some(Key::Other('f')),
        mods: Modifiers::ODILIA,
        repeat: 1,
        consume: true,
        mode: Some(ScreenReaderMode::new("BrowseMode")),
        notify: true,
    };
    add_keybind(ocap, nothing).await;
    add_keybind("h".parse().unwrap(), next_header).await;
    add_keybind(find_in_tree_kb, find_in_tree).await;
    add_keybind("Shift+h".parse().unwrap(), previous_header).await;
    add_keybind("Odilia+b".parse().unwrap(), activate_browse_mode).await;
    add_keybind("Odilia+a".parse().unwrap(), activate_focus_mode).await;

    println!("STARTING ODILIA!");
    //I am trying to fix this by making TTS not be lazily initialised
    TTS.set(Mutex::new(Speaker::new("odilia").unwrap())).unwrap();
    speak_non_interrupt("welcome to odilia!").await;

    let mut rx = create_keybind_channel();
    while let Some(kb) = rx.recv().await {
      // need to do this explicitly for now
      run_keybind_func(&kb).await;
    }
    // get key event listeners set up
    /*
    if let Err(error) = grab_async(keystroke_handler).await {
      println!("Error: {:?}", error);
    }*/
    // Connect to the accessibility bus
    let (_event_loop, conn) = open_a11y_bus().await?;
    println!("{}", conn.unique_name());
    let addr1 = Proxy::new(
        "org.a11y.atspi.Registry",
        "/org/a11y/atspi/registry/deviceeventcontroller",
        TIMEOUT,
        Arc::clone(&conn),
    );
    let registry = Proxy::new(
        "org.a11y.atspi.Registry",
        "/org/a11y/atspi/registry",
        TIMEOUT,
        Arc::clone(&conn),
    );
    
 // Tell at-spi we're interested in focus events
    registry
        .method_call(
            "org.a11y.atspi.Registry",
            "RegisterEvent",
            ("Object:TextCaretMoved\0",)
        )
        .await?;

    let matching = Proxy::new(
        "org.a11y.atspi.DeviceEventController",
        "/org/a11y/atspi/listeners/0",
        TIMEOUT,
        Arc::clone(&conn),
    );
    // Listen for those events
    let mr = MatchRule::new_signal(StateChanged::INTERFACE, StateChanged::NAME);
    let mr2 = MatchRule::new_signal(CaretMoved::INTERFACE, CaretMoved::NAME);
    let mr3 = MatchRule::new_signal("org.a11y.atspi.DeviceEventController", "NotifyListenersSync");
    //let mr3 = mr3.with_path("/org/a11y/atspi/listeners/0");
    // msgmatch must be bound, else we get no events!
    let (_msgmatch, mut stream) = conn.add_match(mr2).await?.msg_stream();

    while let Some(msg) = stream.next().await {
        let mut iter = msg.iter_init();
        println!("{:?}", iter);
        let acc = Accessible::new(
          msg.sender().unwrap(),
          msg.path().unwrap(),
          Arc::clone(&conn)
        );
        let event_type: String = iter.get().unwrap();
        let name = acc.name().await.unwrap();
        let role = acc.localized_role_name().await.unwrap();
        //let sender = msg.sender().unwrap().clone();
        //let path = msg.path().unwrap().clone();
        /*
        println!("{:?}", msg);
        /*
        if event_type != "focused" {
            continue;
        }
        iter.next(); // Done retrieving this String
        let gained_focus = iter.get::<i32>().unwrap() == 1;
        if !gained_focus {
            continue;
        }*/

        // Construct a proxy to the newly focused DBus object
        // I think the only time these unwraps would panic is if we were constructing a
        // message, and it wasn't fully constructed yet, so this *should* be fine
  */
        /*
        let name_fut: MethodReply<String> = accessible.get("org.a11y.atspi.Accessible", "Name");
        let chr_cnt_fut: MethodReply<i32> = accessible.get("org.a11y.atspi.Text", "CharacterCount");
        let role_fut: MethodReply<(String,)> =
            accessible.method_call("org.a11y.atspi.Accessible", "GetLocalizedRoleName", ());
        let attrs_fut: MethodReply<(HashMap<String,String>,)> =
            accessible.method_call("org.a11y.atspi.Accessible", "GetAttributes", ());
        let children_fut: MethodReply<((String, dbus::Path<'static>),)> =
            accessible.method_call("org.a11y.atspi.Accessible", "GetChildAtIndex", (0,));
        let chr_cnt = tokio::try_join!(chr_cnt_fut);
        let children = tokio::try_join!(children_fut).unwrap().0.0;
        let text_fut: MethodReply<(String,)> =
            accessible.method_call("org.a11y.atspi.Text", "GetText", (0, chr_cnt.unwrap().0));
        let index_in_fut: MethodReply<(i32,)> = accessible.method_call("org.a11y.atspi.Accessible", "GetIndexInParent", ());
        let (name, (role,), (attrs,), (text,), (index_in,)) = tokio::try_join!(name_fut, role_fut, attrs_fut, text_fut, index_in_fut)?;
        */
        let attrs = acc.attrs().await;
        println!("{:?}", attrs);
        //println!("<{0}>{1}</{0}>", attrs.get("tag").unwrap(), text);
        /*
        let accessible2 = Proxy::new(
            msg.sender().unwrap(),
            & children.1,
            TIMEOUT,
            Arc::clone(&conn),
        );
        println!("INDEX: {:?}", index_in);
        println!("{:?}", children);
        
        let place_fut: MethodReply<(i32,)> = accessible2.method_call("org.a11y.atspi.Accessible", "GetIndexInParent", ());
        let place = tokio::try_join!(place_fut);
        println!("{:?}", place);
        */
        println!("{}, {}", name, role);
        let text = format!("{}, {}", name, role);
        tokio::task::spawn(speak(text));
    }
    Ok(())
}

/// Opens a connection to the session bus, grabs the address of the a11y bus, and disconnects from
/// the session bus.
async fn a11y_bus_address() -> Result<String, dbus::Error> {
    let (io_res, conn) = dbus_tokio::connection::new_session_sync()?;
    // Run this in the background
    let io_res = tokio::task::spawn(async move {
        let err = io_res.await;
        // Todo: Make this fail gracefully
        panic!("Lost connection to DBus: {}", err);
    });

    let proxy = Proxy::new("org.a11y.Bus", "/org/a11y/bus", TIMEOUT, conn);
    let (address,) = proxy.method_call("org.a11y.Bus", "GetAddress", ()).await?;

    io_res.abort(); // Disconnect from session bus
    Ok(address)
}

/// Connect to the a11y bus.
async fn open_a11y_bus() -> Result<(tokio::task::JoinHandle<()>, Arc<SyncConnection>), dbus::Error>
{
    let addr = a11y_bus_address().await?;
    let mut channel = Channel::open_private(&addr)?;
    channel.register()?;
    let (io_res, conn) = dbus_tokio::connection::from_channel(channel)?;
    // Run this in the background
    let jh = tokio::task::spawn(async move {
        let error = io_res.await;
        // Todo: Make this fail gracefully
        panic!("Lost connection to DBus: {}", error);
    });
    Ok((jh, conn))
}
