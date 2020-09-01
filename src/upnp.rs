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
use std::time::Duration;

use async_std::task;

pub use miniupnpc::commands::Protocol;
pub use miniupnpc::Error;
pub use miniupnpc::ValidIgd;

/// Discover an Internet Gateway Device.
pub async fn discover_igd() -> Result<Option<ValidIgd>, Error> {
    let handle = task::spawn(async {
        miniupnpc::discover(
            Duration::from_secs(2),
            None,
            None,
            miniupnpc::LocalPort::Any,
            false,
            2,
        )
        .map(|list| list.get_valid_igd())
    });

    handle.await
}

/// Add port mapping
#[allow(clippy::too_many_arguments)]
pub async fn add_port_mapping<C, S, D>(
    control_url: C,
    service_type: S,
    ext_port: u16,
    in_port: u16,
    in_client: Ipv4Addr,
    desc: D,
    proto: Protocol,
    lease_duration: Duration,
) -> Result<(), Error>
where
    C: AsRef<str> + Send + 'static,
    S: AsRef<str> + Send + 'static,
    D: AsRef<str> + Send + 'static,
{
    let handle = task::spawn(async move {
        miniupnpc::commands::add_port_mapping(
            control_url,
            service_type,
            ext_port,
            in_port,
            in_client,
            desc,
            proto,
            None,
            lease_duration,
        )
    });

    handle.await
}

pub async fn get_external_ip_address<C, S>(
    control_url: C,
    service_type: S,
) -> Result<Ipv4Addr, Error>
where
    C: AsRef<str> + Send + 'static,
    S: AsRef<str> + Send + 'static,
{
    let handle = task::spawn(async move {
        miniupnpc::commands::get_external_ip_address(control_url, service_type)
    });

    handle.await
}
