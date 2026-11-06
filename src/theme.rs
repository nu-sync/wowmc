use clap::builder::styling::{AnsiColor, Reset, Style, Styles};

pub const HEADING: Style = AnsiColor::Yellow.on_default().bold();
pub const ACCENT: Style = AnsiColor::Cyan.on_default().bold();
pub const SCOPE: Style = AnsiColor::Magenta.on_default();
pub const SUCCESS: Style = AnsiColor::Green.on_default();
pub const WARNING: Style = AnsiColor::Yellow.on_default();
pub const ERROR: Style = AnsiColor::Red.on_default().bold();
pub const MUTED: Style = Style::new().dimmed();
pub const RESET: Reset = Reset;

pub const HELP: Styles = Styles::styled()
    .header(HEADING)
    .usage(HEADING)
    .literal(ACCENT)
    .placeholder(SCOPE)
    .valid(SUCCESS)
    .invalid(WARNING)
    .error(ERROR);
