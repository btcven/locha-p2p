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

use std::net::Ipv4Addr;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;

use async_std::task;

use futures::channel::mpsc::{channel, Receiver};
use futures::future;
use futures::stream::Stream;
use futures::Future;

use log::{error, info, warn};

use crate::sync::TaskInterrupt;

/// UPnP task
///
/// This UPnP task adds a map to the given port on certain timeouts. It only
/// does any work when mapping the port. It also can return a future to resolve
/// our external IPv4 address.
///
/// When no UPnP IGD is found the task will simply finish.
#[derive(Debug)]
pub struct UpnpTask {
    interrupt: TaskInterrupt,
    handle: Option<task::JoinHandle<()>>,
}

impl UpnpTask {
    /// Create a new [`UpnpTask`]
    ///
    /// # Note
    ///
    /// The task _must_ be started using [`UpnpTask::start`].
    pub fn new() -> UpnpTask {
        UpnpTask {
            interrupt: TaskInterrupt::new(),
            handle: None,
        }
    }

    /// Start the [`UpnpTask`].
    ///
    /// # Arguments
    ///
    /// - `port`: the port we want to map using UPnP.
    /// - `discover`: whether we want to discover our external IPv4 address
    /// with UPnP.
    ///
    /// # Return
    ///
    /// This method will return the [`UpnpTask`] handle to the running task
    /// and maybe a [`ResolveAddress`] future if `discover` is true. Once
    /// the address is found [`ResolveAddress`] future will resolve to
    /// the given IPv4 address if found.
    pub fn start(
        &mut self,
        port: u16,
        discover: bool,
    ) -> Option<ResolveAddress> {
        // Create the MSPC channel for the resolution of the external
        // IPv4 address when `discover` is true.
        let (tx, rx) = if discover {
            let chan = channel::<Option<Ipv4Addr>>(1);
            (Some(chan.0), Some(chan.1))
        } else {
            (None, None)
        };

        let interrupt = TaskInterrupt::clone(&self.interrupt);
        let handle = task::spawn(async move {
            let interrupt = interrupt;
            let tx = tx;

            // Discover UPnP devices and retrieve an IGD if found.
            let (urls, data, lanaddr) = match miniupnpc::discover(
                Duration::from_secs(2),
                None,
                None,
                miniupnpc::LocalPort::Any,
                false,
                2,
            )
            .map(|list| list.get_valid_igd())
            {
                Ok(Some(igd)) => {
                    if igd.data.is_none() {
                        error!(target: "locha-p2p", "UPnP: No valid IGDs found");
                        return;
                    }

                    (igd.urls, igd.data.unwrap(), igd.lan_address)
                }
                Ok(None) | Err(_) => {
                    error!(target: "locha-p2p", "UPnP: No valid IGDs found");
                    return;
                }
            };

            // Retrieve External IPv4 address from UPnP IGD
            if discover {
                let ipaddr = miniupnpc::commands::get_external_ip_address(
                    urls.control_url(),
                    data.first().service_type(),
                );

                match ipaddr {
                    Ok(ip) => future::poll_fn({
                        let mut tx = tx.unwrap();
                        move |cx| match tx.poll_ready(cx) {
                            Poll::Ready(Ok(_)) => {
                                tx.start_send(Some(ip)).unwrap();
                                Poll::Ready(())
                            }
                            Poll::Ready(Err(e)) => {
                                if e.is_disconnected() {
                                    warn!(
                                        target: "locha-p2p",
                                        "ResolveAddress future has been dropped"
                                    );
                                }
                                Poll::Ready(())
                            }
                            Poll::Pending => Poll::Pending,
                        }
                    })
                    .await,
                    Err(_) => {
                        error!(target: "locha-p2p", "UPnP: GetExternalIPAddress failed");
                    }
                }
            }

            // Map the given port using AddPortMapping each 20 minutes.
            // This can be stopped using `UpnpTask::stop(self)`.
            loop {
                match miniupnpc::commands::add_port_mapping(
                    urls.control_url(),
                    data.first().service_type(),
                    port,
                    port,
                    lanaddr,
                    "LochaP2P",
                    miniupnpc::commands::Protocol::Tcp,
                    None,
                    Duration::from_millis(0),
                ) {
                    Ok(_) => {
                        info!(
                            target: "locha-p2p",
                            "UPnP: Ports mapped successfully",
                        );
                    }
                    Err(e) => {
                        warn!(
                            target: "locha-p2p",
                            "UPnP: AddPortMapping({}, {}, {}, {}, {}, Tcp, None, 0) failed: {}",
                            urls.control_url(),
                            data.first().service_type(),
                            port, port, lanaddr, e
                        );
                    }
                }

                // Wait 20 minutes or for an interrupt.
                if !interrupt.sleep_for(Duration::from_secs(1200)).await {
                    break;
                }
            }
        });
        self.handle = Some(handle);

        if discover {
            Some(ResolveAddress { rx: rx.unwrap() })
        } else {
            None
        }
    }

    /// Stop the UPnP task
    pub async fn stop(self) {
        if let Some(handle) = self.handle {
            self.interrupt.interrupt().await;
            handle.await;
        }
    }
}

/// A future that will resolve to our external IPv4 address if found.
///
/// This future is created with [`UpnpTask::new`].
///
/// # Return
///
/// - `Some(ipv4)`: if external address was found.
/// - `None`: if external address was not found.
pub struct ResolveAddress {
    rx: Receiver<Option<Ipv4Addr>>,
}

impl Future for ResolveAddress {
    type Output = Option<Ipv4Addr>;

    /// See whether an address was found
    ///
    /// # Panics
    ///
    /// This function will panic if polled againt after `Poll::Ready(_)`.
    fn poll(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Self::Output> {
        match Pin::new(&mut self.rx).poll_next(cx) {
            Poll::Ready(Some(v)) => Poll::Ready(v),
            Poll::Ready(None) => panic!("can't resolve again the future!"),
            Poll::Pending => Poll::Pending,
        }
    }
}
