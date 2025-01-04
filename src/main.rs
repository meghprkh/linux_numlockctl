use std::path::PathBuf;
use std::{env, fs, process::exit, thread::sleep, time::Duration};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum State {
    OFF,
    ON,
}

impl State {
    fn toggled(&self) -> State {
        match self {
            State::OFF => State::ON,
            State::ON => State::OFF,
        }
    }
}

fn get_led_state(path: &PathBuf) -> Option<State> {
    let Ok(brightness_bytes) = fs::read(path) else {
        return None;
    };
    let Ok(state) = String::from_utf8(brightness_bytes.clone()) else {
        return None;
    };
    if state == "0\n" {
        Some(State::OFF)
    } else if state == "1\n" {
        Some(State::ON)
    } else {
        eprintln!(
            "Invalid state {state} in {}. Skipping!",
            path.clone().to_string_lossy()
        );
        None
    }
}

fn get_led_path_and_state() -> (PathBuf, State) {
    const LEDS: &str = "/sys/class/leds/";
    let entries = std::fs::read_dir(LEDS);
    if entries.is_err() {
        eprintln!("Can not open directory {LEDS}");
        exit(1);
    }
    let entries = entries.unwrap();
    for entry in entries {
        let Ok(entry) = entry else {
            continue;
        };
        let file_name_os_str = entry.file_name();
        let Some(file_name) = file_name_os_str.to_str() else {
            continue;
        };
        if !file_name.contains("numlock") {
            continue;
        }
        let Ok(metadata) = entry.metadata() else {
            continue;
        };
        let metadata = if metadata.is_symlink() {
            let Ok(metadata) = fs::metadata(entry.path()) else {
                continue;
            };
            metadata
        } else {
            metadata
        };
        if !metadata.is_dir() {
            continue;
        }
        let brightness_path = entry.path().join("brightness");
        let Some(state) = get_led_state(&brightness_path) else {
            continue;
        };
        return (brightness_path, state);
    }

    eprintln!("Failed to detect num lock state");
    exit(1);
}

fn press_numlock() -> Result<(), uinput::Error> {
    let Ok(device) = uinput::default() else {
        eprintln!("Please run with administrative rights or fix udev rules!");
        exit(1)
    };
    let device = device.name("numlocklrs")?;

    let numlock_key = uinput::event::Keyboard::Key(uinput::event::keyboard::Key::NumLock);
    let device = device.event(numlock_key)?;
    let mut device = device.create()?;

    sleep(Duration::from_micros(100000));

    device.synchronize()?;
    device.press(&numlock_key)?;
    device.synchronize()?;
    device.release(&numlock_key)?;
    device.synchronize()?;

    sleep(Duration::from_micros(100000));

    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() > 2 {
        println!("Invalid usage!");
        exit(1);
    }
    let (path, state) = get_led_path_and_state();
    if args.len() == 1 {
        if state == State::ON {
            println!("Numlock is on");
        } else {
            println!("Numlock is off");
        };
        exit(0)
    }

    #[derive(Debug, Copy, Clone, Eq, PartialEq)]
    enum Action {
        TOGGLE,
        ON,
        OFF,
        INVALID,
    }

    let action = match args[1].to_uppercase().as_str() {
        "ON" => Action::ON,
        "OFF" => Action::OFF,
        "TOGGLE" => Action::TOGGLE,
        _ => Action::INVALID,
    };
    if action == Action::INVALID {
        println!("Invalid usage!");
        exit(1);
    }

    let should_toggle = match action {
        Action::ON => state == State::OFF,
        Action::OFF => state == State::ON,
        Action::TOGGLE => true,
        _ => false,
    };
    if should_toggle {
        press_numlock().unwrap_or_else(|err| {
            eprintln!("Failed with error {err}");
            exit(1)
        });
        let Some(new_state) = get_led_state(&path) else {
            eprintln!("Failed to get new numlock state!");
            exit(1)
        };
        if new_state != state.toggled() {
            eprintln!("Failed to toggle numlock");
        }
    }
}
