#![doc = include_str!("../README.md")]

pub mod clap {
    //! Styling for clap CLI

    use std::io::IsTerminal;

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

    /// Output format arguments for [`clap`]. Use [`Self::determine_format`] for actually resolving the output format
    #[derive(Debug, Clone, Copy, ::clap::Args)]
    #[group(multiple = false)]
    pub struct OutputFormatArg {
        /// Force readable text output
        #[arg(long)]
        pub text: bool,

        /// Force binary output
        #[arg(long)]
        pub binary: bool,
    }

    impl OutputFormatArg {
        /// Determine the output format based on the flags set in this [`OutputFormatArg`]. If no flags are set, returns [`None`]
        pub fn determine_format(self) -> Option<OutputFormat> {
            if self.text {
                Some(OutputFormat::Text)
            } else if self.binary {
                Some(OutputFormat::Binary)
            } else {
                None
            }
        }

        /// Determine the output format based on the flags set in this [`OutputFormatArg`], and whether the output
        /// is a terminal or not.
        pub fn determine_format_with_stream(self, output_stream: &impl IsTerminal) -> OutputFormat {
            debug_assert!(
                !(self.text && self.binary),
                "Both text and binary are set. This is invalid"
            );

            if let Some(format) = self.determine_format() {
                return format;
            }

            if output_stream.is_terminal() {
                // An output stream was given. Use text if terminal because the user is probably reading it. Binary otherwise
                OutputFormat::Text
            } else {
                // Binary for files and other non-terminal stream
                OutputFormat::Binary
            }
        }
    }

    /// The output format, obtained from [`OutputFormatArg::determine_format`]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub enum OutputFormat {
        /// Binary output
        Binary,

        /// Human-readable text output
        Text,
    }
}
