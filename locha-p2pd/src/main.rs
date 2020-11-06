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

use std::io::Write;
use std::path::Path;

use clap::{load_yaml, App};

use async_std::task;

use locha_p2p::identity::Identity;
use locha_p2p::p2p::behaviour;
use locha_p2p::runtime::config::RuntimeConfig;
use locha_p2p::runtime::events::{RuntimeEvents, RuntimeEventsLogger};
use locha_p2p::runtime::Runtime;
use locha_p2p::PeerId;

use log::{error, info};

use rustyline::error::ReadlineError;
use rustyline::Editor;

mod arguments;
use arguments::Arguments;

struct EventsHandler;

impl RuntimeEvents for EventsHandler {
    fn on_new_message(&mut self, peer_id: &PeerId, message: String) {
        let id = peer_id.to_string();
        info!("Message from ...{}:", &id[id.len() - 8..]);
        info!("{}", message);
    }
}

#[async_std::main]
async fn main() {
    env_logger::Builder::new()
        .format(|buf, record| {
            writeln!(buf, "[{}] - {}", record.level(), record.args())
        })
        .filter_level(log::LevelFilter::Info)
        .init();

    let cli_yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(cli_yaml).get_matches();
    let arguments = Arguments::from_matches(&matches);

    let identity = load_identity(&arguments.identity)
        .expect("couldn't load identity file");
    info!("our peer id: {}", identity.id());

    // Reach out to another node if specified
    let mut bootstrap_nodes = Vec::new();
    if !arguments.dont_bootstrap {
        bootstrap_nodes
            .extend_from_slice(behaviour::bootstrap_nodes().as_slice());

        for mut addr in arguments.peers {
            let peer_id = match addr.pop() {
                Some(libp2p::multiaddr::Protocol::P2p(id_hash)) => {
                    PeerId::from_multihash(id_hash).expect("Invalid PeerId")
                }
                _ => {
                    error!(
                        "Supplied invalid peer address, must contain a Peer ID"
                    );
                    return;
                }
            };

            info!("Adding peer {} with address {}", peer_id, addr);
            bootstrap_nodes.push((peer_id, addr));
        }
    }

    let config = RuntimeConfig {
        identity,
        channel_cap: 25,
        heartbeat_interval: 10,
        listen_addr: arguments.listen_addr,

        upnp: !arguments.disable_upnp,
        mdns: !arguments.disable_mdns,

        bootstrap_nodes,
    };

    let (runtime, runtime_task) =
        Runtime::new(config, Box::new(RuntimeEventsLogger::new(EventsHandler)))
            .unwrap();

    task::spawn(runtime_task);

    if !arguments.dont_bootstrap {
        runtime.bootstrap().await;
    }

    let mut rl = Editor::<()>::new();
    rl.load_history(".locha_p2p_history").ok();

    loop {
        match rl.readline(">>> ") {
            Ok(line) => {
                if !line.starts_with("/") && !line.is_empty() {
                    runtime.send_message(line).await;
                } else if !line.is_empty() {
                    rl.add_history_entry(line.as_str());

                    match line.as_str() {
                        "/id" => {
                            println!("Peer ID: {}", runtime.peer_id().await);
                        }
                        "/network_info" => {
                            let info = runtime.network_info().await;

                            println!("Connected peers: {}", info.num_peers);
                            println!(
                                "Total connections: {}",
                                info.num_connections
                            );
                            println!(
                                "Pending connections: {}",
                                info.num_connections_pending
                            );
                            println!(
                                "Connections established: {}",
                                info.num_connections_established
                            );
                        }
                        "/kbuckets" => {
                            let kbuckets = runtime.kbuckets().await;
                            if kbuckets.len() > 0 {
                                for (i, kbucket) in kbuckets.iter().enumerate()
                                {
                                    println!("KBucket {}:", i);

                                    for entry in kbucket {
                                        println!(
                                            "- \"{}\": {:?}",
                                            entry.node.key.preimage(),
                                            entry.node.value
                                        );
                                    }
                                }
                            }
                        }
                        "/external_addresses" => {
                            let addrs = runtime.external_addresses().await;
                            if addrs.len() > 0 {
                                println!("External addresses:");
                                for addr in addrs {
                                    println!("- {}", addr);
                                }
                            }
                        }
                        _ => {
                            println!("Invalid command");
                        }
                    }
                }

                // TODO: handle commands
            }
            Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => break,
            Err(e) => {
                error!(target: "locha-p2p", "readline error: {}", e);
                break;
            }
        }
    }

    info!(target: "locha-p2p", "exiting...");
    runtime.stop().await
}

fn load_identity(file: &Path) -> std::io::Result<Identity> {
    Identity::from_file(file).or_else(|_| {
        let id = Identity::generate();
        id.to_file(file)?;
        Ok(id)
    })
}
