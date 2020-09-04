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

//! # UPnP

use std::collections::HashSet;
use std::net::Ipv4Addr;
use std::time::Duration;

use futures::channel::mpsc::{channel, Receiver, Sender};
use futures::channel::oneshot::channel as oneshot_channel;
use futures::channel::oneshot::Sender as OneshotSender;
use futures::{Future, FutureExt, SinkExt, StreamExt};

use log::error;

pub use miniupnpc::commands::Protocol;

pub const CHECK_PORT_INTERVALS_SECS: u64 = 1200;

#[derive(Debug, Clone)]
pub struct Upnp {
    tx: Sender<UpnpEvent>,
}

impl Upnp {
    pub fn new() -> (Upnp, impl Future<Output = ()> + Send + 'static) {
        let (tx, rx) = channel(5);
        (Upnp { tx }, task(rx))
    }

    pub async fn get_external_ip_address(&self) -> Option<Ipv4Addr> {
        let (tx, rx) = oneshot_channel();
        self.tx
            .clone()
            .send(UpnpEvent::GetExternalIPAddress(tx))
            .await
            .unwrap();
        rx.await.unwrap()
    }

    pub async fn add_port_mapping(
        &self,
        desc: String,
        proto: Protocol,
        port: u16,
    ) {
        self.tx
            .clone()
            .send(UpnpEvent::AddPortMapping(PortMap { desc, proto, port }))
            .await
            .unwrap()
    }

    pub async fn stop(&self) {
        self.tx.clone().send(UpnpEvent::Stop).await.unwrap()
    }
}

#[derive(Debug)]
enum UpnpEvent {
    Stop,
    GetExternalIPAddress(OneshotSender<Option<Ipv4Addr>>),
    AddPortMapping(PortMap),
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
struct PortMap {
    desc: String,
    proto: Protocol,
    port: u16,
}

async fn task(mut rx: Receiver<UpnpEvent>) {
    let mut port_mappings: HashSet<PortMap> = HashSet::new();
    let mut interval = async_std::stream::interval(Duration::from_secs(
        CHECK_PORT_INTERVALS_SECS,
    ));

    loop {
        futures::select_biased! {
            event = rx.next().fuse() => {
                let event = event.unwrap_or(UpnpEvent::Stop);

                match event {
                    UpnpEvent::GetExternalIPAddress(tx) => {
                        match discover() {
                            Ok(Some((urls, igd, _))) => {
                                match miniupnpc::commands::get_external_ip_address(
                                    urls.control_url(),
                                    igd.first().service_type(),
                                ) {
                                    Ok(ip) => tx.send(Some(ip)).ok(),
                                    Err(_) => tx.send(None).ok(),
                                }
                            }
                            Ok(None) | Err(_) => tx.send(None).ok(),
                        };
                    }
                    UpnpEvent::AddPortMapping(map) => {
                        if port_mappings.get(&map).is_none() {
                            port_mappings.insert(map.clone());
                            match discover() {
                                Ok(Some((urls, igd, lanaddr))) => {
                                    miniupnpc::commands::add_port_mapping(
                                        urls.control_url(),
                                        igd.first().service_type(),
                                        map.port,
                                        map.port,
                                        lanaddr,
                                        map.desc,
                                        map.proto,
                                        None,
                                        Duration::from_secs(0),
                                    )
                                    .ok();
                                }
                                Ok(None) | Err(_) => {}
                            }
                        }
                    }
                    UpnpEvent::Stop => return,
                };
            },
            _ = interval.next().fuse() => {
                // If any port mapping, remap them because we don't know
                // if the map was kept on the UPnP IGD, so... for reliability
                // just remap again.
                //
                // This might actually help in the case the IGD is reset, or
                // any other reason. Doing this each 20 mins (1200 s) does not
                // hurts performance.
                if port_mappings.len() > 0 {
                    match discover() {
                        Ok(Some((urls, igd, lanaddr))) => {
                            for map in port_mappings.iter() {
                                miniupnpc::commands::add_port_mapping(
                                    urls.control_url(),
                                    igd.first().service_type(),
                                    map.port,
                                    map.port,
                                    lanaddr,
                                    &map.desc,
                                    map.proto,
                                    None,
                                    Duration::from_secs(0),
                                )
                                .ok();
                            }
                        }
                        Ok(None) | Err(_) => (),
                    };
                }
            }
        }
    }
}

fn discover() -> Result<
    Option<(miniupnpc::Urls, miniupnpc::IgdData, Ipv4Addr)>,
    miniupnpc::Error,
> {
    match miniupnpc::discover(
        Duration::from_secs(2),
        None,
        None,
        miniupnpc::LocalPort::Any,
        false,
        2,
    )
    .map(|list| list.get_valid_igd())?
    {
        Some(igd) => {
            if igd.data.is_none() {
                error!(target: "locha-p2p", "UPnP: No valid IGDs found");
                return Ok(None);
            }

            Ok(Some((igd.urls, igd.data.unwrap(), igd.lan_address)))
        }
        None => Ok(None),
    }
}
