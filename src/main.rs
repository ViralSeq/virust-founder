use clap::Parser;
use virust_founder::{ cli::{ Args, Commands }, founder::founder };

fn main() {
    let args = Args::parse();
    match args.command {
        Commands::Run { input, output, keep_original } => {
            if let Err(_e) = founder(input, output, keep_original) {
                // TODO! any top level error handling?  Write to .error file?
            }
        }
    }
}
