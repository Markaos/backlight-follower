use std::sync::mpsc::{Sender, channel};
use std::thread;
use crate::{timer, Backlight};
use crate::updater::OutputMessage::{Alarm, ChangeBrightness};
use std::process::Command;

pub struct OutputManager {
    tx: Sender<OutputMessage>
}

enum OutputMessage {
    ChangeBrightness(f32),
    Alarm,
}

impl OutputManager {
    pub fn new(output: Backlight) -> OutputManager {
        let (tx, rx) = channel();
        let manager = OutputManager {
            tx: tx.clone()
        };

        thread::spawn(move || {
            let timer = timer::Timer::new(move || {
                tx.send(Alarm).unwrap();
            });

            let mut brightness = 0;
            loop {
                match rx.recv() {
                    Ok(message) => {
                        match message {
                            ChangeBrightness(value) => {
                                brightness = level_to_raw(value, &output);
                                timer.set(100);
                            }

                            Alarm => {
                                let cmd = format!(
                                    "{} {}",
                                    output.update_command.as_ref().expect("Output needs update command"),
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

        manager
    }

    pub fn set_brightness(&self, brightness: f32) {
        self.tx.send(ChangeBrightness(brightness)).unwrap();
    }
}

fn level_to_raw(level: f32, output: &Backlight) -> i32 {
    (level * output.max as f32 + output.min as f32) as i32
}