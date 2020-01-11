use std::sync::mpsc::{channel, Sender};
use std::thread;
use std::time::Duration;

use crate::timer::TimerMessage::SetTimer;

enum TimerMessage {
    SetTimer(u64),
}

pub struct Timer {
    internal_tx: Sender<TimerMessage>,
}

impl Timer {

    pub fn new(handler: impl Fn() + 'static + Send) -> Timer {
        let (tx, rx) = channel();
        let timer = Timer {
            internal_tx: tx,
        };

        thread::spawn(move || {
            loop {
                let mut time = 0;
                match rx.recv() {
                    Ok(message) => match message {
                        TimerMessage::SetTimer(dur) => {
                            time = dur;
                        }
                    }
                    _ => {}
                }
                loop {
                    match rx.recv_timeout(Duration::from_millis(time as u64)) {
                        Ok(message) => match message {
                            TimerMessage::SetTimer(dur) => {
                                time = dur;
                                continue;
                            }
                        }

                        _ => {
                            // Timeout error - time to call the handler
                            handler();
                            break;
                        }
                    }
                }
            }
        });

        timer
    }

    pub fn set(&self, time_ms: u64) {
        self.internal_tx.send(SetTimer(time_ms)).unwrap();
    }
}