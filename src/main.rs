extern crate inotify;

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::process::Command;
use std::str::FromStr;
use std::sync::Arc;
use std::sync::mpsc::channel;
use std::thread;

use inotify::{
    Inotify,
    WatchMask,
};

use crate::Message::ChangeBrightness;

mod timer;
mod conf;

pub struct Backlight {
    min: i32,
    max: i32,
    path: Option<String>,
    update_command: Option<String>,
}

enum Message {
    ChangeBrightness(i32),
    Alarm,
}

fn main() {
    let (input, output_raw) = conf::parse_conf("/etc/backlight-follower.conf");

    let output = Arc::new(output_raw);
    let output_cloned = output.clone();

    let (tx, rx) = channel();
    let cloned_tx = tx.clone();
    thread::spawn(move || {
        let timer = timer::Timer::new(move || {
            cloned_tx.send(Message::Alarm).unwrap();
        });

        let mut brightness = 0;
        loop {
            match rx.recv() {
                Ok(message) => {
                    match message {
                        Message::ChangeBrightness(value) => {
                            brightness = value;
                            timer.set(100);
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