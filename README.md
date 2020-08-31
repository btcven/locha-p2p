# locha-p2p

<p align="center">
  <a href="https://locha.io/">
    <img height="200px" src="./logo.png">
  </a>
  <br>
  <a href="https://travis-ci.com/btcven/locha-mesh-chat">
    <img src="https://travis-ci.com/btcven/locha-mesh-chat.svg?branch=master" title="Build Status">
  </a>
</p>
<p align="center">
  <a href="https://locha.io/">Project Website</a> |
  <a href="https://locha.io/donate">Donate</a> |
  <a href="https://github.com/sponsors/rdymac">Sponsor</a> |
  <a href="https://locha.io/buy">Buy</a>
</p>

<h1 align="center">Locha P2P</h1>

This repository contains the building blocks for the P2P chat functionality
written in the [Rust] programming language.

[Rust]: https://www.rust-lang.org/

# <a href="https://btcven.github.io/locha-p2p/locha_p2p/index.html">Documentation</a>

# Dependencies

- Rust >=1.43
- OpenSSL development libraries
- pkg-config
- GCC, clang or MSVC compiler.
- libclang

Installation of dependencies on Ubuntu 18.04/20.04:

```bash
sudo apt install gcc pkg-config libssl-dev libclang-9-dev
```

It's recommended to install `rustc` and `cargo` from your distribution package
manager. On Ubuntu 18.04/20.04:


```bash
sudo apt install rustc cargo
```

However it's fine to use [rustup.rs] to have a working toolchain if your
distribution `rustc` version is too outdated.

[rustup.rs]: https://rustup.rs/

# License

Copyright (c) 2020 Bitcoin Venezuela and Locha Mesh Developers.

Licensed under the Apache License, Version 2.0

Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the License for the specific language governing permissions and limitations under the License.

Read the full text: [LICENSE](./LICENSE)