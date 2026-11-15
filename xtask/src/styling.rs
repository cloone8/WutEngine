//! Styling for the [`clap`] command-line output

use clap::builder::Styles;
use clap::builder::styling::AnsiColor;
use clap::builder::styling::Effects;
use clap::builder::styling::Style;

/// Section headers in the help output
const HEADER: Style = AnsiColor::Green.on_default().effects(Effects::BOLD);

/// The usage line
const USAGE: Style = AnsiColor::Green.on_default().effects(Effects::BOLD);

/// Literal text the user types, such as flags and subcommand names
const LITERAL: Style = AnsiColor::Cyan.on_default().effects(Effects::BOLD);

/// Placeholders for values the user fills in
const PLACEHOLDER: Style = AnsiColor::Cyan.on_default();

/// The "error:" label
const ERROR: Style = AnsiColor::Red.on_default().effects(Effects::BOLD);

/// Valid values and suggestions in errors
const VALID: Style = AnsiColor::Cyan.on_default().effects(Effects::BOLD);

/// The invalid value in errors
const INVALID: Style = AnsiColor::Yellow.on_default().effects(Effects::BOLD);

/// Styling for [`clap`]
pub(crate) const STYLING: Styles = Styles::styled()
    .header(HEADER)
    .usage(USAGE)
    .literal(LITERAL)
    .placeholder(PLACEHOLDER)
    .error(ERROR)
    .valid(VALID)
    .invalid(INVALID);

/// The template to use for the long version (`--version`) command.
/// TODO: Probably always resolves to the variables of _this_ crate, not the caller. Should fix that somehow
pub(crate) const LONG_VERSION: &str = concat!(
    env!("CARGO_PKG_VERSION"),
    "\n\nMade by:\n",
    env!("CARGO_PKG_AUTHORS")
);
