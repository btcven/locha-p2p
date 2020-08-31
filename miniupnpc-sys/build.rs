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
use std::process::Command;
use std::str::FromStr;

// miniupnpd: VERSION 2.1
const OID: &str = "582375b64f347d6ef1a4dc3478ff3678ea923f80";

fn main() {
    let out_dir = std::env::var("OUT_DIR")
        .map(|s| PathBuf::from_str(s.as_str()).unwrap())
        .expect("OUT_DIR not found!");

    let repo_dir = out_dir.join("miniupnp");

    let repo = match git2::Repository::clone(
        "https://github.com/miniupnp/miniupnp",
        &repo_dir,
    ) {
        Ok(r) => r,
        Err(e) => {
            if e.raw_class() == 3 && e.raw_code() == -4 {
                git2::Repository::open(&repo_dir).unwrap()
            } else {
                panic!("could not clone repository: {}", e);
            }
        }
    };

    let oid = git2::Oid::from_str(OID).unwrap();
    let commit = repo
        .find_commit(oid)
        .expect("could not find miniupnpc version on repository");

    // Create new branch for repository pointing at the given commit
    let _branch = repo.branch(OID, &commit, false);

    // Checkout tree
    let rev = format!("refs/heads/{}", OID);
    let obj = repo
        .revparse_single(rev.as_ref())
        .expect("could not parse revision");
    repo.checkout_tree(&obj, None)
        .expect("could not checkout tree");
    repo.set_head(rev.as_ref()).expect("could not set HEAD");

    let miniupnpc_dir = repo_dir.join("miniupnpc");

    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap();
    let target_family = std::env::var("CARGO_CFG_TARGET_FAMILY").unwrap();

    let update_res =
        Command::new(miniupnpc_dir.join("updateminiupnpcstrings.sh"))
            .current_dir(&miniupnpc_dir)
            .output()
            .unwrap();
    assert!(update_res.status.success());

    let mut build = cc::Build::new();

    build.warnings(true);

    build.file(miniupnpc_dir.join("miniwget.c"));
    build.file(miniupnpc_dir.join("minixml.c"));
    build.file(miniupnpc_dir.join("igd_desc_parse.c"));
    build.file(miniupnpc_dir.join("minisoap.c"));
    build.file(miniupnpc_dir.join("miniupnpc.c"));
    build.file(miniupnpc_dir.join("upnpreplyparse.c"));
    build.file(miniupnpc_dir.join("upnpcommands.c"));
    build.file(miniupnpc_dir.join("upnperrors.c"));
    build.file(miniupnpc_dir.join("connecthostport.c"));
    build.file(miniupnpc_dir.join("portlistingparse.c"));
    build.file(miniupnpc_dir.join("receivedata.c"));
    build.file(miniupnpc_dir.join("upnpdev.c"));

    if target_family == "unix" {
        build
            .file(miniupnpc_dir.join("minissdpc.c"))
            .define("MINIUPNPC_SET_SOCKET_TIMEOUT", None)
            .define("MINIUPNPC_GET_SRC_ADDR", None)
            .define("_BSD_SOURCE", None)
            .define("_DEFAULT_SOURCE", None);

        if target_os == "netbsd" {
            build.define("_NETBSD_SOURCE", None);
        }

        if target_os == "freebsd" || target_os == "macos" {
            build.define("_XOPEN_SOURCE", "600");
        }
    } else if target_family == "windows" {
        build.file(miniupnpc_dir.join("minissdpc.c"));
        build.define("NDEBUG", None).define("_WIN32_WINNT", "0X501");
        let compiler = build.get_compiler();
        if compiler.is_like_gnu() || compiler.is_like_clang() {
            build.flag("-lws2_32");
            build.flag("-liphlpapi");
            build.pic(true);
        }

        if compiler.is_like_msvc() {
            build.define("_CRT_SECURE_NO_WARNINGS", None);
        }
    }

    build.cargo_metadata(true);

    build.compile("miniupnpc");

    let bindings = bindgen::builder()
        .header(miniupnpc_dir.join("miniupnpc.h").to_str().unwrap())
        .header(miniupnpc_dir.join("upnpcommands.h").to_str().unwrap())
        .header(miniupnpc_dir.join("upnperrors.h").to_str().unwrap())
        .generate()
        .expect("could not generate bindings");

    bindings
        .write_to_file(out_dir.join("bindings.rs"))
        .expect("could not write bindings");
}
