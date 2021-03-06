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
    pub peers: Vec<Multiaddr>,
    pub identity: PathBuf,

    pub disable_upnp: bool,
    pub disable_mdns: bool,
    pub dont_bootstrap: bool,
}

impl Arguments {
    /// Construct the arguments from the command line matches.
    pub fn from_matches(matches: &ArgMatches) -> Arguments {
        let listen_addr = value_t!(matches.value_of("listen-addr"), Multiaddr)
            .unwrap_or_else(|e| e.exit());
        let peers = match values_t!(matches.values_of("add-peer"), Multiaddr) {
            Ok(d) => d,
            Err(e) if e.kind == clap::ErrorKind::ArgumentNotFound => vec![],
            Err(e) => e.exit(),
        };
        let identity = value_t!(matches.value_of("identity"), PathBuf)
            .unwrap_or_else(|e| e.exit());

        Arguments {
            listen_addr,
            peers,
            identity,

            disable_upnp: matches.is_present("disable-upnp"),
            disable_mdns: matches.is_present("disable-mdns"),
            dont_bootstrap: matches.is_present("dont-bootstrap"),
        }
    }
}
