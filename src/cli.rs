use clap::builder::styling::{ AnsiColor, Color };
use clap::builder::styling::{ Style, Styles };
use clap::{ ColorChoice, Parser, Subcommand };

pub const BANNER: &str =
    "\x1b[0;91m███████  ██████  ██    ██ ███    ██ ██████  ███████ ██████  \x1b[0m\n\
     \x1b[0;93m██      ██    ██ ██    ██ ████   ██ ██   ██ ██      ██   ██ \x1b[0m\n\
     \x1b[0;92m█████   ██    ██ ██    ██ ██ ██  ██ ██   ██ █████   ██████  \x1b[0m\n\
     \x1b[0;96m██      ██    ██ ██    ██ ██  ██ ██ ██   ██ ██      ██   ██ \x1b[0m\n\
     \x1b[0;95m██       ██████   ██████  ██   ████ ██████  ███████ ██   ██ \x1b[0m\n\
     \x1b[0;90m                 F O U N D E R   P I P E L I N E               \x1b[0m\n";

#[derive(Parser, Debug, Clone)]
#[command(
    name = "Founder pipeline",
    version = env!("CARGO_PKG_VERSION"),
    about = BANNER,
    color = ColorChoice::Always,
    styles = get_styles()
)]
pub struct Args {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
    /// Run the Founder pipeline
    #[command(alias = "r")]
    Run {
        /// Input directory path
        #[arg(short, long)]
        input: String,

        /// Output directory path
        #[arg(short, long)]
        output: String,

        /// keep original files
        #[arg(long, default_value_t = false)]
        keep_original: bool,
    },
}

pub fn get_styles() -> Styles {
    Styles::styled()
        .usage(
            Style::new()
                .bold()
                .underline()
                .fg_color(Some(Color::Ansi(AnsiColor::Yellow)))
        )
        .header(
            Style::new()
                .bold()
                .underline()
                .fg_color(Some(Color::Ansi(AnsiColor::Yellow)))
        )
        .literal(Style::new().fg_color(Some(Color::Ansi(AnsiColor::Green))))
        .invalid(
            Style::new()
                .bold()
                .fg_color(Some(Color::Ansi(AnsiColor::Red)))
        )
        .error(
            Style::new()
                .bold()
                .fg_color(Some(Color::Ansi(AnsiColor::Red)))
        )
        .valid(
            Style::new()
                .bold()
                .underline()
                .fg_color(Some(Color::Ansi(AnsiColor::Green)))
        )
        .placeholder(Style::new().fg_color(Some(Color::Ansi(AnsiColor::White))))
}
