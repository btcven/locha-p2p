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

mod events;

pub use self::events::{Event, Events};

use std::borrow::Cow;
use std::io;

use termion::input::MouseTerminal;
use termion::raw::{IntoRawMode, RawTerminal};
use termion::screen::AlternateScreen;
use tui::backend::TermionBackend;
use tui::layout::*;
use tui::style::*;
use tui::text::*;
use tui::widgets::*;

use locha_p2p::runtime::NetworkInfo;

pub type Terminal = tui::Terminal<
    TermionBackend<AlternateScreen<MouseTerminal<RawTerminal<io::Stdout>>>>,
>;

pub fn build_terminal() -> io::Result<Terminal> {
    let stdout = io::stdout().into_raw_mode()?;
    let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    let stdout = TermionBackend::new(stdout);

    tui::Terminal::new(stdout)
}

#[derive(Debug)]
pub struct App {
    tabs: TabsInfo,
    network_info: NetInfo,
}

impl App {
    pub fn new(info: NetworkInfo) -> App {
        App {
            tabs: TabsInfo::new(vec!["Main".into()]),
            network_info: NetInfo::new(info),
        }
    }

    pub fn update_network_info(&mut self, info: NetworkInfo) {
        self.network_info.info = info;
    }

    pub fn draw(&mut self, terminal: &mut Terminal) -> io::Result<()> {
        terminal.draw(|f| {
            let size = f.size();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    [
                        Constraint::Length(3),
                        Constraint::Min(6),
                        Constraint::Min(5),
                    ]
                    .as_ref(),
                )
                .split(size);

            let block = Block::default()
                .style(Style::default().bg(Color::Black).fg(Color::White));
            f.render_widget(block, size);

            f.render_widget(self.tabs.widget(), chunks[0]);

            f.render_widget(self.network_info.draw(), chunks[1]);
        })
    }
}

#[derive(Debug)]
pub struct TabsInfo {
    titles: Vec<Cow<'static, str>>,
    index: usize,
}

impl TabsInfo {
    fn new(titles: Vec<Cow<'static, str>>) -> TabsInfo {
        TabsInfo { titles, index: 0 }
    }

    fn next(&mut self) {
        self.index = (self.index + 1) % self.titles.len();
    }

    fn titles(&self) -> Vec<Spans> {
        self.titles
            .iter()
            .map(|v| Spans::from(vec![Span::raw(v.as_ref())]))
            .collect()
    }

    fn widget(&self) -> Tabs {
        Tabs::new(self.titles())
            .block(Block::default().borders(Borders::ALL).title("Tabs"))
            .select(self.index)
            .style(Style::default().fg(Color::White))
            .highlight_style(
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .bg(Color::Black),
            )
    }
}

#[derive(Debug)]
pub struct NetInfo {
    info: NetworkInfo,
}

impl NetInfo {
    fn new(info: NetworkInfo) -> NetInfo {
        NetInfo { info }
    }

    fn draw(&self) -> Paragraph {
        let text = vec![
            Spans::from(vec![
                Span::raw("Peers connected: "),
                Span::styled(
                    self.info.num_peers.to_string(),
                    Style::default()
                        .add_modifier(Modifier::BOLD)
                        .fg(Color::Green),
                ),
            ]),
            Spans::from(vec![
                Span::raw("Total connections: "),
                Span::styled(
                    self.info.num_connections.to_string(),
                    Style::default()
                        .add_modifier(Modifier::BOLD)
                        .fg(Color::Green),
                ),
            ]),
            Spans::from(vec![
                Span::raw("Total pending connections: "),
                Span::styled(
                    self.info.num_connections_pending.to_string(),
                    Style::default()
                        .add_modifier(Modifier::BOLD)
                        .fg(Color::Yellow),
                ),
            ]),
            Spans::from(vec![
                Span::raw("Total established connections: "),
                Span::styled(
                    self.info.num_connections_established.to_string(),
                    Style::default()
                        .add_modifier(Modifier::BOLD)
                        .fg(Color::Green),
                ),
            ]),
        ];

        Paragraph::new(text)
            .block(
                Block::default()
                    .title("Network information")
                    .borders(Borders::ALL),
            )
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: false })
    }
}

#[cfg(test)]
pub mod test {
    use super::*;

    #[test]
    fn test_build_terminal() {
        build_terminal().unwrap();
    }
}
