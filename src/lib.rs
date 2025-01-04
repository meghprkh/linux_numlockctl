//! Easy way to trigger and get state of numlock in linux

use std::{fs, io, path::PathBuf, thread::sleep, time::Duration};

/// Error type
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Unexpected IO error '{0}'")]
    IoError(#[from] io::Error),
    #[error("Failed to interpret LED state with path '{0}'")]
    InvalidLedState(PathBuf),
    #[error("Unexpected UInput Error '{0}'")]
    UInputError(#[from] uinput::Error),
    #[error("No valid LEDs found")]
    NoLedsFound,
    #[error("Failed to change numlock state")]
    FailedToPressNumlock,
}

/// Result type alias
pub type Res<T> = std::result::Result<T, Error>;

/// State - ON/OFF
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum State {
    OFF,
    ON,
}

impl State {
    pub fn toggled(&self) -> State {
        match self {
            State::OFF => State::ON,
            State::ON => State::OFF,
        }
    }
}

/// Gets the led state of a numlock led given its path
pub fn get_led_state(path: &PathBuf) -> Res<State> {
    let brightness_bytes = fs::read(path)?;
    let Ok(state) = String::from_utf8(brightness_bytes.clone()) else {
        return Err(Error::InvalidLedState(path.clone()));
    };
    if state == "0\n" {
        Ok(State::OFF)
    } else if state == "1\n" {
        Ok(State::ON)
    } else {
        return Err(Error::InvalidLedState(path.clone()));
    }
}

/// Gets a numlock led path and its state
pub fn get_led_path_and_state() -> Res<(PathBuf, State)> {
    const LEDS: &str = "/sys/class/leds/";
    let entries = std::fs::read_dir(LEDS)?;
    for entry in entries {
        let entry = entry?;
        let file_name_os_str = entry.file_name();
        let file_name = file_name_os_str.to_string_lossy();
        if !file_name.contains("numlock") {
            continue;
        }
        let metadata = entry.metadata()?;
        let metadata = if metadata.is_symlink() {
            fs::metadata(entry.path())?
        } else {
            metadata
        };
        if !metadata.is_dir() {
            continue;
        }
        let brightness_path = entry.path().join("brightness");
        let state = get_led_state(&brightness_path)?;
        return Ok((brightness_path, state));
    }

    Err(Error::NoLedsFound)
}

/// Emulates press of a numlock key. If `led_path_and_state` is provided, it is checked if the
/// state got toggled.
pub fn press_numlock(led_path_and_state: Option<(&PathBuf, State)>) -> Res<()> {
    let device = uinput::default()?;
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

    if let Some((path, state)) = led_path_and_state {
        for _ in 0..10 {
            sleep(Duration::from_micros(10000));
            if get_led_state(&path)? != state {
                return Ok(());
            }
        }
        Err(Error::FailedToPressNumlock)
    } else {
        sleep(Duration::from_micros(100000));
        Ok(())
    }
}
