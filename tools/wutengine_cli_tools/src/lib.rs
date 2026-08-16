#![doc = include_str!("../README.md")]

pub mod clap {
    //! Styling for clap CLI

    use clap::builder::Styles;
    use clap::builder::styling::AnsiColor;
    use clap::builder::styling::Effects;
    use clap::builder::styling::Style;

    const HEADER: Style = AnsiColor::Green.on_default().effects(Effects::BOLD);
    const USAGE: Style = AnsiColor::Green.on_default().effects(Effects::BOLD);
    const LITERAL: Style = AnsiColor::Cyan.on_default().effects(Effects::BOLD);
    const PLACEHOLDER: Style = AnsiColor::Cyan.on_default();
    const ERROR: Style = AnsiColor::Red.on_default().effects(Effects::BOLD);
    const VALID: Style = AnsiColor::Cyan.on_default().effects(Effects::BOLD);
    const INVALID: Style = AnsiColor::Yellow.on_default().effects(Effects::BOLD);

    /// Styling for [clap]
    pub const STYLING: Styles = Styles::styled()
        .header(HEADER)
        .usage(USAGE)
        .literal(LITERAL)
        .placeholder(PLACEHOLDER)
        .error(ERROR)
        .valid(VALID)
        .invalid(INVALID);
}
