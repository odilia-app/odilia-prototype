use std::{sync::Arc, time::Duration};

use dbus::{
    channel::Channel,
    message::{MatchRule, SignalArgs},
    nonblock::{Proxy, MethodReply, SyncConnection, stdintf::org_freedesktop_dbus::Properties},
};
use futures::stream::StreamExt;
use once_cell::sync::OnceCell;
//use once_cell::sync::Lazy;
use tokio::sync::Mutex;

use atspi_codegen::event::OrgA11yAtspiEventObjectStateChanged as StateChanged;
use tts_subsystem::{Priority, Speaker};

const TIMEOUT: Duration = Duration::from_secs(1);

// Create a lazily initialised static Speaker
// The closure is called when `TTS` is first used, and its return value is used to initialise a
// hidden OnceCell in this static
static TTS:OnceCell<Mutex<Speaker>>=OnceCell::new();
//static TTS: Lazy<Mutex<Speaker>> = Lazy::new(|| Mutex::new(Speaker::new("yggdrasil").unwrap()));

async fn speak(text: impl AsRef<str>) {
    // We can use it directly here, it will automatically be initialised if necessary
    let temp = TTS.get().unwrap().lock().await;
    temp.cancel().unwrap();
    temp.speak(Priority::Important, text.as_ref()).unwrap();
}

#[tokio::main]
async fn main() -> Result<(), dbus::Error> {
    //I am trying to fix this by making TTS not be lazily initialised
    TTS.set(Mutex::new(Speaker::new("yggdrasil").unwrap())).unwrap();
    // Connect to the accessibility bus
    let (_event_loop, conn) = open_a11y_bus().await?;
    // Create a proxy object that interacts with the at-spi registry
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
            ("Object:StateChanged:Focused\0",),
        )
        .await?;

    // Listen for those events
    let mr = MatchRule::new_signal(StateChanged::INTERFACE, StateChanged::NAME);
    // msgmatch must be bound, else we get no events!
    let (_msgmatch, mut stream) = conn.add_match(mr).await?.msg_stream();
        while let Some(msg) = stream.next().await {
            let mut iter = msg.iter_init();
            let event_type: String = iter.get().unwrap();
            if event_type != "focused" {
                continue;
            }
            iter.next(); // Done retrieving this String
            let gained_focus = iter.get::<i32>().unwrap() == 1;
            if !gained_focus {
                continue;
            }
            
            // Construct a proxy to the newly focused DBus object
            // I think the only time these unwraps would panic is if we were constructing a
            // message, and it wasn't fully constructed yet, so this *should* be fine
            let accessible = Proxy::new(msg.sender().unwrap(), msg.path().unwrap(), TIMEOUT, Arc::clone(&conn));
            let name_fut: MethodReply<String> = accessible.get("org.a11y.atspi.Accessible", "Name");
            let role_fut: MethodReply<(String,)> = accessible.method_call("org.a11y.atspi.Accessible", "GetLocalizedRoleName", ());
            let (name, (role,)) = tokio::try_join!(name_fut, role_fut)?;
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
