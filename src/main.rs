use std::{sync::Arc, time::Duration, collections::HashMap};

use atspi::Accessible;

use dbus::{
    channel::Channel,
    message::{MatchRule, SignalArgs},
    nonblock::{stdintf::org_freedesktop_dbus::Properties, MethodReply, Proxy, SyncConnection},
};
use futures::stream::StreamExt;
use once_cell::sync::OnceCell;
//use once_cell::sync::Lazy;
use tokio::sync::Mutex;

use atspi_codegen::event::OrgA11yAtspiEventObjectStateChanged as StateChanged;
use atspi_codegen::event::OrgA11yAtspiEventObjectTextCaretMoved as CaretMoved;
use atspi_codegen::device_event_controller::OrgA11yAtspiDeviceEventController;
use tts_subsystem::{Priority, Speaker};

const TIMEOUT: Duration = Duration::from_secs(1);

// Create a lazily initialised static Speaker
// The closure is called when `TTS` is first used, and its return value is used to initialise a
// hidden OnceCell in this static
static TTS: OnceCell<Mutex<Speaker>> = OnceCell::new();
//static TTS: Lazy<Mutex<Speaker>> = Lazy::new(|| Mutex::new(Speaker::new("yggdrasil").unwrap()));

async fn speak(text: impl AsRef<str>) {
    // We can use it directly here, it will automatically be initialised if necessary
    let temp = TTS.get().unwrap().lock().await;
    temp.cancel().unwrap();
    temp.speak(Priority::Important, text.as_ref()).unwrap();
}

#[tokio::main]
async fn main() -> Result<(), dbus::Error> {
    println!("STARTING YGGDRASIL!");
    //I am trying to fix this by making TTS not be lazily initialised
    TTS.set(Mutex::new(Speaker::new("yggdrasil").unwrap()))
        .unwrap();
    // Connect to the accessibility bus
    let (_event_loop, conn) = open_a11y_bus().await?;
    // Create a proxy object that interacts with the at-spi registry
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
            ("Object:TextCaretChanged\0",)
        )
        .await?;

    let nv: Vec<(i32, i32, &str, i32)> = vec![];
    let success = addr1.method_call(
        "org.a11y.atspi.DeviceEventController",
        "RegisterKeystrokeListener",
        (dbus::Path::from("/org/a11y/atspi/listeners/0"),
         vec![(43, 0x68, "h", 0)],
         0,
         3,
         (false, false, true))).await?;
    println!("{:?}", success);
    // Listen for those events
    let mr = MatchRule::new_signal(StateChanged::INTERFACE, StateChanged::NAME);
    let mr2 = MatchRule::new_signal(CaretMoved::INTERFACE, CaretMoved::NAME);
    // msgmatch must be bound, else we get no events!
    let (_msgmatch, mut stream) = conn.add_match(mr2).await?.msg_stream();
    while let Some(msg) = stream.next().await {
        let mut iter = msg.iter_init();
        let event_type: String = iter.get().unwrap();
        //let sender = msg.sender().unwrap().clone();
        //let path = msg.path().unwrap().clone();
        /*
        let acc = Accessible::new(
          sender,
          path,
          Arc::clone(&conn)
        );*/
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
        let accessible = Accessible::with_timeout(
            msg.sender().unwrap(),
            msg.path().unwrap(),
            Arc::clone(&conn),
            TIMEOUT,
        );
        println!("{:?}", accessible.localized_role_name().await);
        accessible.children(false).await.unwrap().for_each(|a| async {
            println!("{:?}", a.unwrap().localized_role_name().await);
        });
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
        println!("<{0}>{1}</{0}>", attrs.get("tag").unwrap(), text);
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
        //let text = format!("{}, {}", name, role);
        //tokio::task::spawn(speak(text));
        */
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
