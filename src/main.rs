use atspi::accessible::AccessibleExt;
use atspi::events::*;
use atspi::Role;
use once_cell::sync::OnceCell;
use std::sync::Mutex;
use tts_subsystem::Priority;
use tts_subsystem::Speaker;
fn speak(text: &str) {
    let temp = TTS.get().unwrap().lock().unwrap();
    temp.cancel().unwrap();
    temp.speak(Priority::Important, text).unwrap();
}
static TTS: OnceCell<Mutex<Speaker>> = OnceCell::new();
fn main() {
    if let Err(e) = atspi::init() {
        eprintln!("Error initialising libatspi: {}", e);
        std::process::exit(1);
    }
    TTS.set(Mutex::new(Speaker::new("yggdrasil").unwrap()));
    let listener = EventListener::new(|e| {
        let source = e.source().unwrap();
        speak(&handle_component(source));
    });
    listener.register("object:state-changed:focused").unwrap();
    Event::main();
    atspi::exit();
}

fn handle_component(source: atspi::prelude::Accessible) -> String {
    let name = source.name();
    let text = source.text();

    let role = source.role().unwrap();
    let spoken_control = {
        if name.is_err() && text.is_none() {
            "unlabeled".to_owned()
        } else if name.is_err() {
            text.unwrap().to_string()
        } else if text.is_none() {
            name.unwrap().to_string()
        } else {
            format!("{}, {}", name.unwrap(), text.unwrap())
        }
    };

    let spoken_role = handle_component_kind(role);
    format!("{}: {}", spoken_control, spoken_role)
}

fn handle_component_kind(role: Role) -> &'static str {
    match role {
        Role::Invalid => "invalid component",
        Role::AcceleratorLabel => "accelerator",
        Role::Alert => "alert",
        Role::Animation => "animation",
        Role::Arrow => "arrow",
        Role::Calendar => "calendar controll",
        Role::Canvas => "canvas",
        Role::CheckBox => "checkbox",
        Role::CheckMenuItem => "check menu item",
        Role::ColorChooser => "color picker",
        Role::ColumnHeader => "column header",
        Role::ComboBox => "combo box",
        Role::DateEditor => "date picker",
        Role::DesktopIcon => "desktop icon",
        Role::DesktopFrame => "desktop",
        Role::Dial => "dialer",
        Role::Dialog => "dialog",
        Role::DirectoryPane => "directory panel",
        Role::DrawingArea => "drawing zone",
        Role::FileChooser => "file picker",
        Role::Filler => "filler",
        Role::FocusTraversable => "focus traversable",
        Role::FontChooser => "font picker",
        Role::Frame => "frame",
        Role::GlassPane => "transparent pannel",
        Role::HtmlContainer => "html container",
        Role::Icon => "icon",
        Role::Image => "grafic",
        Role::InternalFrame => "internal frame",
        Role::Label => "static text",
        Role::LayeredPane => "layered layout",
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
        Role::TearoffMenuItem => "tare off menu item",
        Role::Terminal => "terminal",
        Role::Text => "text area",
        Role::ToggleButton => "toggle",
        Role::ToolBar => "tool bar",
        Role::ToolTip => "tool tip",
        Role::Tree => "treeview",
        Role::TreeTable => "tree table",
        Role::Unknown => "unknown",
        Role::Viewport => "view",
        Role::Window => "window",
        Role::Extended => "extended",
        Role::Header => "header",
        Role::Footer => "footer",
        Role::Paragraph => "paragraph",
        Role::Ruler => "ruler",
        Role::Application => "application",
        Role::Autocomplete => "autocomplete",
        Role::Editbar => "edit bar",
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
        Role::Rating => "raiting",
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
        Role::ContentDeletion => "deleted",
        Role::ContentInsertion => "inserted",
        Role::Mark => "marked content",
        Role::Suggestion => "suggestion",
        Role::LastDefined => "last defined",
        Role::__Unknown(_) => "",
        _ => "",
    }
}
