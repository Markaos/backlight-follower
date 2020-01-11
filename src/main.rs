extern crate inotify;

use inotify::{
    WatchMask,
    Inotify,
};

use std::sync::Arc;
use std::thread;
use std::process::Command;
use std::str::FromStr;
use std::fs::File;
use std::io::{Seek, SeekFrom, Read};
use std::sync::mpsc::channel;
use std::time::Duration;
use crate::Message::ChangeBrightness;

struct Backlight {
    min: i32,
    max: i32,
    path: Option<String>,
    update_command: Option<String>,
}

enum Message {
    ChangeBrightness(i32),
    SetTimer(i32),
    Alarm,
}

fn read_backlight(file: &mut File) -> i32 {
    let mut buffer: Vec<u8> = Vec::new();

    file.seek(SeekFrom::Start(0))
        .expect("Couldn't seek");
    file.read_to_end(&mut buffer)
        .expect("Couldn't read input file");

    buffer.pop(); // Remove \n

    let string = String::from_utf8(buffer)
        .expect("Failed to parse the file");

    return i32::from_str(string.as_str()).expect("Not a number...");
}

fn convert_levels(level: i32, input: &Backlight, output: &Backlight) -> i32 {
    return (
        (level as f32 - input.min as f32) / (input.max - input.min) as f32
            * output.max as f32 + output.min as f32
    ) as i32
}

fn main() {
    let input = Backlight{
        min: 1,
        max: 49,
        path: Option::from(String::from("/sys/class/backlight/acpi_video0/brightness")),
        update_command: Option::None,
    };

    let output_raw = Backlight{
        min: 0,
        max: 100,
        path: Option::None,
        update_command: Option::from(String::from("ddcutil -b 7 setvcp 10")),
    };

    let output = Arc::new(output_raw);
    let output_cloned = output.clone();

    let (tx, rx) = channel();
    let cloned_tx = tx.clone();
    thread::spawn(move || {
        let (timer_tx, timer_rx) = channel();
        thread::spawn(move || {
            loop {
                let mut time = 0;
                match timer_rx.recv() {
                    Ok(message) => match message {
                        Message::SetTimer(dur) => {
                            time = dur;
                        }
                        _ => {}
                    }
                    _ => {}
                }
                loop {
                    match timer_rx.recv_timeout(Duration::from_millis(time as u64)) {
                        Ok(message) => match message {
                            Message::SetTimer(dur) => {
                                time = dur;
                                continue;
                            }
                            _ => {}
                        }

                        _ => {
                            cloned_tx.send(Message::Alarm).unwrap();
                            break;
                        }
                    }
                }
            }
        });
        let mut brightness = 0;
        loop {
            match rx.recv() {
                Ok(message) => {
                    match message {
                        Message::ChangeBrightness(value) => {
                            brightness = value;
                            timer_tx.send(Message::SetTimer(100)).unwrap();
                        }

                        Message::Alarm => {
                            let cmd = format!(
                                "{} {}",
                                output_cloned.update_command.as_ref().expect("Output needs update command"),
                                brightness
                            );

                            Command::new("sh")
                                .arg("-c")
                                .arg(cmd)
                                .status()
                                .expect("Something went wrong with the output update command");
                        }
                        _ => { /* WHAT??? */ }
                    }
                }
                Err(_e) => {}
            }
        }
    });

    let path = input.path.as_ref().expect("Input backlight doesn't have a path");
    let mut file = File::open(&path)
        .expect("Couldn't open the file");
    let mut inotify = Inotify::init()
        .expect("Failed to initialize inotify");

    inotify.add_watch(&path, WatchMask::MODIFY)
        .expect("Couldn't initialize watching");

    let mut buffer = [0u8; 4096];
    loop {
        let events = inotify.read_events_blocking(&mut buffer)
            .expect("Reading inotify events failed somehow");

        for _event in events {
            let output_value = convert_levels(read_backlight(&mut file), &input, &output);
            tx.send(ChangeBrightness(output_value)).unwrap();
        }
    }
}
