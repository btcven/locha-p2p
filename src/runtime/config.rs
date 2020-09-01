// Copyright 2020 Locha Inc
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

//! # Runtime configuration

use libp2p::Multiaddr;

use crate::discovery::DiscoveryConfig;
use crate::identity::Identity;

#[derive(Debug)]
pub struct RuntimeConfig {
    pub identity: Identity,
    pub listen_addr: Multiaddr,
    pub channel_cap: usize,
    pub heartbeat_interval: u64,

    pub discovery: DiscoveryConfig,
}
