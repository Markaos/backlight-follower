extern crate inotify;

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::str::FromStr;

use inotify::{
    Inotify,
    WatchMask,
};

mod timer;
mod conf;
mod updater;

pub struct Backlight {
    min: i32,
    max: i32,
    path: Option<String>,
    update_command: Option<String>,
}

fn main() {
    let (input, output) = conf::parse_conf("/etc/backlight-follower.conf");
    let output_manager = updater::OutputManager::new(output);

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
            let output_value = level_from_input(read_backlight(&mut file), &input);
            output_manager.set_brightness(output_value);
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

fn level_from_input(level: i32, input: &Backlight) -> f32 {
    (level as f32 - input.min as f32) / (input.max - input.min) as f32
}