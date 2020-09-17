// Copyright 2020 Bitcoin Venezuela and Locha Mesh Developers
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

use futures::channel::mpsc::{channel, Receiver};
use futures::{SinkExt, StreamExt};

use wasm_timer::Delay;

use termion::input::TermRead;

#[derive(Debug)]
pub enum Event {
    Key(termion::event::Key),
    Tick,
}

#[derive(Debug)]
pub struct Events {
    rx: Receiver<Event>,
}

impl Events {
    pub fn new(tick_rate: Duration) -> Events {
        let (tx, rx) = channel::<Event>(100);

        async_std::task::spawn({
            let tx = tx.clone();
            async move {
                let mut tx = tx;
                let stdin = io::stdin();
                for evt in stdin.keys() {
                    if let Ok(k) = evt {
                        if let Err(_) = tx.send(Event::Key(k)).await {
                            return;
                        }
                    }
                }
            }
        });

        async_std::task::spawn(async move {
            let mut tx = tx;
            let tick_rate = tick_rate;
            let mut delay = Delay::new(tick_rate);

            loop {
                delay.await.unwrap();

                if let Err(_) = tx.send(Event::Tick).await {
                    return;
                }
                delay = Delay::new(tick_rate);
            }
        });

        Events { rx }
    }

    pub async fn next(&mut self) -> Event {
        self.rx.next().await.unwrap()
    }
}
