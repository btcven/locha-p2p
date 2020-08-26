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

//! # Locha P2P
//!
//! This library contains the necessary code to set up a node that works
//! with the Locha P2P Chat.

#![recursion_limit = "1024"]
#![doc(html_logo_url = "https://locha.io/i/128.png")]
#![doc(html_favicon_url = "https://locha.io/i/128.png")]

mod chat_service;
mod config;
mod error;
mod events;
mod identity;
mod sync_start_cond;

pub use self::chat_service::{ChatService, CHAT_SERVICE_GOSSIP_PROTCOL_NAME};
pub use self::config::ChatServiceConfig;
pub use self::error::Error;
pub use self::events::ChatServiceEvents;
pub use self::identity::Identity;
