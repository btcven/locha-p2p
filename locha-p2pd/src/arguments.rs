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

use std::path::PathBuf;

use clap::{value_t, values_t, ArgMatches};

use libp2p::Multiaddr;

/// Command line arguments
#[derive(Debug)]
pub struct Arguments {
    pub listen_addr: Multiaddr,
    pub dials: Vec<Multiaddr>,
    pub identity: PathBuf,
    pub echo: bool,
    pub tui: bool,
}

impl Arguments {
    /// Construct the arguments from the command line matches.
    pub fn from_matches(matches: &ArgMatches) -> Arguments {
        let listen_addr = value_t!(matches.value_of("listen-addr"), Multiaddr)
            .unwrap_or_else(|e| e.exit());
        let dials = match values_t!(matches.values_of("dial"), Multiaddr) {
            Ok(d) => d,
            Err(e) if e.kind == clap::ErrorKind::ArgumentNotFound => vec![],
            Err(e) => e.exit(),
        };
        let identity = value_t!(matches.value_of("identity"), PathBuf)
            .unwrap_or_else(|e| e.exit());
        let echo = matches.is_present("echo");
        let tui = matches.is_present("tui");

        Arguments {
            listen_addr,
            dials,
            identity,
            echo,
            tui,
        }
    }
}
