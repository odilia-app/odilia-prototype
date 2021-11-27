#[macro_use]
extern crate lazy_static;

use odilia_input::{
  init as input_init,EventAction,AsyncFn
};
use odilia_common::{
  input::KeyBinding
};
use std::{sync::Arc, time::Duration, collections::HashMap, future::Future};
use rdev::{Event as RDevEvent, EventType, Key};

use atspi::Accessible;

use dbus::{
    channel::Channel,
    message::{MatchRule, SignalArgs},
    nonblock::{stdintf::org_freedesktop_dbus::Properties, MethodReply, Proxy, SyncConnection},
};
use futures::stream::StreamExt;
use once_cell::sync::OnceCell;
use tokio::sync::{Mutex, mpsc};
use std::thread;

use atspi_codegen::event::OrgA11yAtspiEventObjectStateChanged as StateChanged;
use atspi_codegen::event::OrgA11yAtspiEventObjectTextCaretMoved as CaretMoved;
use atspi_codegen::device_event_controller::OrgA11yAtspiDeviceEventController;
use tts_subsystem::{Priority, Speaker};

const TIMEOUT: Duration = Duration::from_secs(1);


//initialise a global tts object
static TTS: OnceCell<Mutex<Speaker>> = OnceCell::new();

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

/*
Creating a function to turn an async function into the AsyncFn type from odilia-input
*/
async fn boxit<F,T>(func: F) -> AsyncFn
where
  F: Fn() -> T + Send + 'static + Sync,
  T: Future<Output=()> + Send + 'static,
{
  Box::new(move || {Box::new(Box::pin(func()))})
}

fn rdev_event_bs(ev: &RDevEvent) -> EventAction {
  EventAction::Passthrough
}

#[tokio::main]
async fn main() -> Result<(), dbus::Error> {
    let mut kmap: HashMap<KeyBinding, AsyncFn> = HashMap::new();

    kmap.insert("h".parse().unwrap(), boxit(next_header).await);
    kmap.insert("Odilia+f".parse().unwrap(), boxit(find_in_tree).await);
    kmap.insert("Shift+h".parse().unwrap(), boxit(previous_header).await);

    println!("STARTING ODILIA!");
    //I am trying to fix this by making TTS not be lazily initialised
    TTS.set(Mutex::new(Speaker::new("yggdrasil").unwrap())).unwrap();

    input_init(rdev_event_bs, kmap);
    // get key event listeners set up
    /*
    if let Err(error) = grab_async(keystroke_handler).await {
      println!("Error: {:?}", error);
    }*/
    speak_non_interrupt("welcome to odilia!").await;
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
