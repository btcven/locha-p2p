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

use std::cmp::{Ord, Ordering};
use std::io;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use termion::raw::{IntoRawMode, RawTerminal};
use termion::screen::AlternateScreen;
use tui::backend::TermionBackend;
use tui::layout::*;
use tui::style::*;
use tui::text::*;
use tui::widgets::*;
use tui::Terminal;

use unicode_width::UnicodeWidthStr;

use parking_lot::RwLock;

mod events;
pub use self::events::{Event, Events};

#[derive(Clone, Eq, PartialEq)]
pub struct Message {
    timestamp: Duration,
    contents: String,
}

pub struct Data {
    input: String,
    messages: Vec<Message>,
}

impl Data {
    pub fn new() -> Data {
        Data {
            input: String::new(),
            messages: Vec::new(),
        }
    }

    pub fn input_push(&mut self, v: char) {
        self.input.push(v);
    }

    pub fn input_backspace(&mut self) {
        self.input.pop();
    }

    pub fn input_send(&mut self) -> String {
        let msg = Message {
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap(),
            contents: self.input.clone(),
        };
        self.messages.push(msg);
        let ret = self.input.clone();
        self.input.clear();
        ret
    }
}

pub struct UserInterface {
    terminal:
        Terminal<TermionBackend<AlternateScreen<RawTerminal<io::Stdout>>>>,
    ui_data: Arc<RwLock<Data>>,
}

impl UserInterface {
    pub fn new(ui_data: Arc<RwLock<Data>>) -> io::Result<UserInterface> {
        let stdout = io::stdout().into_raw_mode()?;
        let stdout = AlternateScreen::from(stdout);
        let stdout = TermionBackend::new(stdout);
        let terminal = Terminal::new(stdout)?;

        Ok(UserInterface { terminal, ui_data })
    }

    pub fn tick(&mut self) -> io::Result<()> {
        self.terminal.draw({
            let ui_data = self.ui_data.clone();

            move |f| {
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(2)
                    .constraints(
                        [
                            Constraint::Length(1),
                            Constraint::Length(3),
                            Constraint::Min(1),
                        ]
                        .as_ref(),
                    )
                    .split(f.size());

                let msg = vec![
                    Span::raw("Press "),
                    Span::styled(
                        "Enter",
                        Style::default().add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" to send the message"),
                ];

                let mut text = Text::from(Spans::from(msg));
                text.patch_style(Style::default());
                let help_message = Paragraph::new(text);
                f.render_widget(help_message, chunks[0]);

                let ui_data = ui_data.read();
                let input = Paragraph::new(ui_data.input.as_str())
                    .style(Style::default().fg(Color::Yellow))
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title("Message input"),
                    );
                f.render_widget(input, chunks[1]);

                f.set_cursor(
                    chunks[1].x + ui_data.input.width() as u16 + 1,
                    chunks[1].y + 1,
                );

                let mut messages: Vec<Message> = ui_data.messages.clone();
                let messages: Vec<ListItem> = messages
                    .iter()
                    .map(|m| {
                        let content = vec![Spans::from(Span::raw(format!(
                            "[{}]: {}",
                            m.timestamp.as_secs(),
                            m.contents
                        )))];
                        ListItem::new(content)
                    })
                    .collect();
                let messages = List::new(messages).block(
                    Block::default().borders(Borders::ALL).title("Messages"),
                );
                f.render_widget(messages, chunks[2]);
            }
        })?;

        Ok(())
    }
}
