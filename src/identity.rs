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

use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

use libp2p::identity::{secp256k1, Keypair};
use libp2p::PeerId;

/// Length of a secp256k1 Secret Key in bytes
pub const SECRET_KEY_LENGTH: usize = 32;

/// The identity of a single node.
#[derive(Debug, Clone)]
pub struct Identity {
    keypair: secp256k1::Keypair,
}

impl Identity {
    /// Generate a random identity.
    ///
    /// # Example
    ///
    /// ```rust
    /// use locha_p2p::Identity;
    ///
    /// let id = Identity::generate();
    /// ```
    pub fn generate() -> Identity {
        Identity {
            keypair: secp256k1::Keypair::generate(),
        }
    }

    /// Save `Identity` to a file
    ///
    /// # Example
    ///
    /// ```rust
    /// use locha_p2p::Identity;
    ///
    /// let id = Identity::generate();
    /// id.to_file("id.key")
    ///     .expect("couldn't save Identity to a file");
    /// ```
    pub fn to_file<P: AsRef<Path>>(&self, path: P) -> io::Result<()> {
        let mut file = fs::File::create(path)?;
        // TODO: encrypt
        file.write_all(&self.keypair.secret().to_bytes())?;
        Ok(())
    }

    /// Load `Identity` from file
    ///
    /// # Example
    ///
    /// ```rust
    /// use locha_p2p::Identity;
    ///
    /// let id = Identity::from_file("id.key")
    ///     .expect("coudln't load Identity");
    /// ```
    pub fn from_file<P: AsRef<Path>>(path: P) -> io::Result<Identity> {
        let mut file = fs::File::open(path)?;
        let mut bytes = [0u8; SECRET_KEY_LENGTH];
        file.read_exact(&mut bytes)?;

        secp256k1::SecretKey::from_bytes(bytes)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
            .map(Self::from)
    }

    /// Get the PeerId from this `Identity`.
    pub fn id(&self) -> PeerId {
        PeerId::from(self.keypair().public())
    }

    /// Get the Keypair from this `Identity`.
    pub fn keypair(&self) -> Keypair {
        Keypair::Secp256k1(self.keypair.clone())
    }
}

impl From<secp256k1::SecretKey> for Identity {
    /// Generate an identity from a secp256k1 secret key.
    fn from(v: secp256k1::SecretKey) -> Identity {
        Identity {
            keypair: secp256k1::Keypair::from(v),
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use std::env::temp_dir;

    #[test]
    fn generate_ok() {
        let id = Identity::generate();
        let _peer_id = id.id();
        let _keypair = id.keypair();
    }

    #[test]
    fn save_to_file() {
        let mut dir = temp_dir();
        dir.push("id.key");

        Identity::generate().to_file(dir).unwrap()
    }
}
