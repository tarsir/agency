use agency::basic_operation;
use agency::cli::Cli;
use clap::Parser;
use std::io;

fn main() -> io::Result<()> {
    let cli = Cli::parse();

    basic_operation()?;
    Ok(())
}
