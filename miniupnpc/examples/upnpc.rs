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
// limitations under the License

use std::time::Duration;

use clap::{crate_authors, crate_version, value_t, App, Arg};

fn main() {
    let matches = App::new("upnpc")
        .version(crate_version!())
        .author(crate_authors!())
        .about("UPNP client tool")
        .arg(
            Arg::with_name("version")
                .long("version")
                .takes_value(false)
                .help("Print version information"))
        .arg(
            Arg::with_name("ipv6")
                .long("ipv6")
                .short("6")
                .takes_value(false)
                .help("Use IPv6"),
        )
        .arg(
            Arg::with_name("ttl")
                .long("ttl")
                .takes_value(true)
                .default_value("2")
                .help("Sets the value of the socket option `IP_MULTICAST_TTL`/`IPV6_MULTICAST_HOPS` depending on the protocol used."),
        )
        .get_matches();

    if matches.is_present("version") {
        println!("unpnc {}\n", crate_version!());
        println!("miniupnpc API version: {}", miniupnpc::api_version());
        println!("miniupnpc version: {}", miniupnpc::version());
        return;
    }

    let devices = miniupnpc::discover(
        Duration::from_secs(2),
        None,
        None,
        miniupnpc::LocalPort::Any,
        matches.is_present("ipv6"),
        value_t!(matches.value_of("ttl"), u8).unwrap_or_else(|e| e.exit()),
    )
    .unwrap();

    println!("List of UPnP devices found on the network:");
    for device in devices.iter() {
        println!("desc: {}", device.desc_url());
        println!("scope id: {}", device.scope_id());
    }

    let igd = match devices.get_valid_igd() {
        Some(v) => v,
        None => {
            println!("No UPnP devices found");
            return;
        }
    };

    if igd.data.is_some() {
        println!("Found Internet Gateway Device");
    } else {
        println!("Internet Gateway Device not found");
        return;
    }

    let igd_data = igd.data.unwrap();

    display_infos(&igd.urls, &igd_data)
}

fn display_infos(urls: &miniupnpc::Urls, igd_data: &miniupnpc::IgdData) {
    let type_info = miniupnpc::commands::get_connection_type_info(
        urls.control_url(),
        igd_data.first().service_type(),
    )
    .unwrap();

    let status_info = miniupnpc::commands::get_status_info(
        urls.control_url(),
        igd_data.first().service_type(),
    )
    .unwrap();

    let ipaddr = miniupnpc::commands::get_external_ip_address(
        urls.control_url(),
        igd_data.first().service_type(),
    )
    .unwrap();

    println!("Connection type: {}", type_info);
    println!(
        "Status: {}, uptime={}s, LastConnectionError=\"{}\"",
        status_info.status,
        status_info.uptime.as_secs(),
        status_info.lastconnerror
    );
    println!("External IPv4 address: {}", ipaddr);
}
