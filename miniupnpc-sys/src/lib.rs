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

//! # miniupnp-sys - MiniUPnP Raw Bindings
//!
//! These are the low level bindings for MiniUPnP.
//!
//! **Note:** The miniupnp is compiled from source code and linked
//! statically, it doesn't use the installed library on your OS.
//! Consider opening a PR is you consider it necessary.

#![allow(non_snake_case)]
#![doc(html_logo_url = "https://locha.io/i/128.png")]
#![doc(html_favicon_url = "https://locha.io/i/128.png")]
#![allow(clippy::redundant_static_lifetimes)]
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
