// Copyright 2020 Bitcoin Venezuela and Locha Mesh Developers
//
// Based on tui-rs syncrhonous event handling. Ported to async-std
// and removed non-needed code.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::io;
use std::time::Duration;

use async_std::sync::{channel, Receiver};
use async_std::task;

use termion::event::Key;
use termion::input::TermRead;

pub enum Event {
    Input(Key),
    Tick,
}

/// A small event handler that wrap termion input and tick events. Each event
/// type is handled in its own thread and returned to a common `Receiver`
pub struct Events {
    rx: Receiver<Event>,
}

impl Events {
    pub fn new(tick_rate: Duration) -> Events {
        let (tx, rx) = channel::<Event>(10);
        task::spawn({
            let tx = tx.clone();
            async move {
                let stdin = io::stdin();
                for evt in stdin.keys() {
                    if let Ok(key) = evt {
                        tx.send(Event::Input(key)).await
                    }
                }
            }
        });
        task::spawn(async move {
            loop {
                tx.send(Event::Tick).await;
                task::sleep(tick_rate).await;
            }
        });

        Events { rx }
    }

    pub async fn next(&self) -> Result<Event, async_std::sync::RecvError> {
        self.rx.recv().await
    }
}
