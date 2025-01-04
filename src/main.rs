use clap::{Parser, Subcommand};
use std::process::exit;

use linux_numlockctl as numlockctl;

#[derive(Debug, Clone, Copy, Subcommand, Eq, PartialEq)]
enum Command {
    #[command(about = "Print status of numlock [Default]")]
    STATUS,
    #[command(about = "Toggle numlock")]
    TOGGLE,
    #[command(about = "Switch on numlock")]
    ON,
    #[command(about = "Switch off numlock")]
    OFF,
}

#[derive(Debug, Parser)] // requires `derive` feature
#[command(name = "numlockctl")]
#[command(bin_name = "numlockctl")]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

fn main() {
    let args = Cli::parse();
    let command = args.command.unwrap_or(Command::STATUS);

    let (path, state) = numlockctl::get_led_path_and_state().unwrap_or_else(|err| {
        eprintln!("Failed to get numlock led state: {err}");
        exit(1)
    });
    if command == Command::STATUS {
        if state == numlockctl::State::ON {
            println!("Numlock is on");
        } else {
            println!("Numlock is off");
        };
        exit(0)
    }

    let should_toggle = match command {
        Command::ON => state == numlockctl::State::OFF,
        Command::OFF => state == numlockctl::State::ON,
        Command::TOGGLE => true,
        _ => false,
    };
    if should_toggle {
        numlockctl::press_numlock(Some((&path, state))).unwrap_or_else(|err| {
            eprintln!("Failed to press numlock with error: {err}");
            exit(1)
        });
    }
}
