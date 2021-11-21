use std::{sync::Arc, time::Duration};

use dbus::{
    channel::Channel,
    message::{MatchRule, SignalArgs},
    nonblock::{Proxy, SyncConnection},
};
use futures::stream::StreamExt;
use once_cell::sync::OnceCell;
//use once_cell::sync::Lazy;
use atspi::Accessible;
use atspi_codegen::event::OrgA11yAtspiEventObjectStateChanged as StateChanged;
use tokio::sync::Mutex;
use tts_subsystem::{Priority, Speaker};

const TIMEOUT: Duration = Duration::from_secs(1);

static TTS: OnceCell<Mutex<Speaker>> = OnceCell::new();

async fn speak(text: impl AsRef<str>) {
    // We can use it directly here, it will automatically be initialised if necessary
    let temp = TTS.get().unwrap().lock().await;
    temp.cancel().unwrap();
    temp.speak(Priority::Important, text.as_ref()).unwrap();
}

#[tokio::main]
async fn main() -> Result<(), dbus::Error> {
    //I am trying to fix this by making TTS not be lazily initialised
    TTS.set(Mutex::new(Speaker::new("yggdrasil").unwrap()))
        .unwrap();
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

        let accessible = Accessible::new(
            msg.sender().unwrap(),
            msg.path().unwrap(),
            Arc::clone(&conn),
        );
        /*
        let name_fut: MethodReply<String> = accessible.get("org.a11y.atspi.Accessible", "Name");
        let role_fut: MethodReply<(String,)> =
            accessible.method_call("org.a11y.atspi.Accessible", "GetLocalizedRoleName", ());
        let (name, (role,)) = tokio::try_join!(name_fut, role_fut)?;
        */
        let name = accessible.name().await.unwrap();
        let role = accessible.role().await.unwrap();
        handle_role(role);
        let text = format!("{}, {}", name, handle_role(role));
        tokio::task::spawn(speak(text));
    }
    Ok(())
}
type Role = atspi::enums::AtspiRole;
fn handle_role(role: atspi::enums::AtspiRole) -> &'static str {
    match role {
        Role::Invalid => "invalid component",
        Role::AcceleratorLable => "accelerator",
        Role::Alert => "alert",
        Role::Animation => "animation",
        Role::Arrow => "arrow",
        Role::Calendar => "calendar controll",
        Role::Canvas => "canvas",
        Role::Checkbox => "checkbox",
        Role::CheckMenuItem => "check menu item",
        Role::ColorChooser => "color picker",
        Role::ColumnHeader => "column header",
        Role::ComboBox => "combo box",
        Role::DateEditor => "date picker",
        Role::DesktopIcon => "desktop icon",
        Role::DesktopFrame => "desktop",
        Role::Dile => "dialer",
        Role::Dialog => "dialog",
        Role::DirectoryPane => "directory panel",
        Role::DrawingArea => "drawing zone",
        Role::FileChooser => "file picker",
        Role::Filler => "filler",
        Role::FocusTraversable => "focus traversable",
        Role::FontChooser => "font picker",
        Role::Frame => "frame",
        Role::GlassPane => "transparent pannel",
        Role::HTMLContainer => "html container",
        Role::Icon => "icon",
        Role::Image => "grafic",
        Role::InternalFrame => "internal frame",
        Role::Label => "static text",
        Role::LayerdPane => "layered layout",
        Role::List => "list",
        Role::ListItem => "list item",
        Role::Menu => "menu",
        Role::MenuBar => "menu bar",
        Role::MenuItem => "menu item",
        Role::OptionPane => "option selector",
        Role::PageTab => "tab control",
        Role::PageTabList => "tab list",
        Role::Panel => "panel layout",
        Role::PasswordText => "secure text box",
        Role::PopupMenu => "popup",
        Role::ProgressBar => "progress bar",
        Role::PushButton => "button",
        Role::RadioButton => "radio button",
        Role::RadioMenuItem => "radio menu item",
        Role::RootPane => "root container",
        Role::RowHeader => "row header",
        Role::ScrollBar => "scroll widget",
        Role::ScrollPane => "scrollable panel",
        Role::Separator => "separator",
        Role::Slider => "slider",
        Role::SpinButton => "spinner",
        Role::SplitPane => "split panel",
        Role::StatusBar => "status bar",
        Role::Table => "table",
        Role::TableCell => "table cel",
        Role::TableColumnHeader => "column header",
        Role::TableRowHeader => "row header",
        Role::TearOffMenuItem => "tare off menu item",
        Role::Terminal => "terminal",
        Role::Text => "text area",
        Role::ToggleButton => "toggle",
        Role::ToolBar => "tool bar",
        Role::ToolTip => "tool tip",
        Role::Tree => "treeview",
        Role::TreeTable => "tree table",
        Role::Unknown => "unknown",
        Role::ViewPort => "view",
        Role::Window => "window",
        Role::Extended => "extended",
        Role::Header => "header",
        Role::Footer => "footer",
        Role::Paragraph => "paragraph",
        Role::Ruler => "ruler",
        Role::Application => "application",
        Role::AutoComplete => "autocomplete",
        Role::EditBar => "edit bar",
        Role::Embedded => "embedded object",
        Role::Entry => "edit box",
        Role::Chart => "chart",
        Role::Caption => "caption",
        Role::DocumentFrame => "document container",
        Role::Heading => "title",
        Role::Page => "page",
        Role::Section => "section",
        Role::RedundantObject => "",
        Role::Form => "form control",
        Role::Link => "link",
        Role::InputMethodWindow => "input method dialog",
        Role::TableRow => "row",
        Role::TreeItem => "treeview item",
        Role::DocumentSpreadsheet => "spreadsheet",
        Role::DocumentPresentation => "presentation",
        Role::DocumentText => "text",
        Role::DocumentWeb => "webview component",
        Role::DocumentEmail => "email",
        Role::Comment => "comment",
        Role::ListBox => "list box",
        Role::Grouping => "group",
        Role::ImageMap => "image map",
        Role::Notification => "notification",
        Role::InfoBar => "info",
        Role::LevelBar => "level bar",
        Role::TitleBar => "title",
        Role::BlockQuote => "block quote",
        Role::Audio => "audioplayer",
        Role::Video => "videoplayer",
        Role::Definition => "definition",
        Role::Article => "article",
        Role::Landmark => "landmark",
        Role::Log => "log",
        Role::Marquee => "marquee",
        Role::Math => "math area",
        Role::Raiting => "raiting",
        Role::Timer => "timer controll",
        Role::Static => "static",
        Role::MathFraction => "fraction",
        Role::MathRoot => "root",
        Role::Subscript => "subscript",
        Role::Superscript => "superscript",
        Role::DescriptionList => "description list",
        Role::DescriptionTerm => "description term",
        Role::DescriptionValue => "description value",
    Role::Footnote => "footnote",
        Role::ContentDeletion => "      deleted",
        Role::ContentInsertion => "inserted",
        Role::Mark => "marked content",
        Role::Suggestion => "suggestion",
        Role::LastDefined => "last defined",
        Role::Unknown => "unknown",
        _ => "",
    }
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
