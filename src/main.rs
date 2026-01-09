use clap::Parser;
use virust_founder::{ cli::{ Args, Commands }, founder::founder };

fn main() {
    let args = Args::parse();
    match args.command {
        Commands::Run { input, output, keep_original, use_phanghorn } => {
            if let Err(e) = founder(input, output, keep_original, use_phanghorn) {
                eprintln!("Error: {e}");
            }
        }
    }
}
