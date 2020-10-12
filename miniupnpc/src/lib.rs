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

//! # MiniUPnP
//!
//! This crate contains safe bindings to the [miniupnpc] library.
//!
//! >"UPnP is a set of networking protocols that allows networked devices
//! to seamlessy discover each other's presence on the network."
//!
//! See also: [Universal Plug and Play](https://en.wikipedia.org/wiki/Universal_Plug_and_Play)
//!
//! [miniupnpc]: https://github.com/miniupnp/miniupnp

#![doc(html_logo_url = "https://locha.io/i/128.png")]
#![doc(html_favicon_url = "https://locha.io/i/128.png")]

use std::{error, fmt, ptr};

use std::borrow::Cow;
use std::ffi::{CStr, CString};
use std::marker::PhantomData;
use std::mem::MaybeUninit;
use std::os::raw::{c_char, c_int, c_uchar};

use std::net::{Ipv4Addr, SocketAddrV4};
use std::str::FromStr;
use std::time::Duration;

/// MiniUPnP API version
pub const fn api_version() -> u32 {
    miniupnpc_sys::MINIUPNPC_API_VERSION
}

/// MiniUPnP version
pub fn version() -> &'static str {
    CStr::from_bytes_with_nul(miniupnpc_sys::MINIUPNPC_VERSION)
        .unwrap()
        .to_str()
        .unwrap()
}

/// MiniUPnP library errors.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Error(c_int);

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let str: &'static CStr =
            unsafe { CStr::from_ptr(miniupnpc_sys::strupnperror(self.0)) };
        write!(f, "{}", str.to_string_lossy())
    }
}

impl error::Error for Error {}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum LocalPort {
    /// The OS assigns the source port
    Any,
    /// Uses the same destination port (1900).
    Same,
    /// Other local port.
    Other(u16),
}

/// Discover uPnP devices.
///
/// This function blocks until all devices have been discovered.
/// `delay` impacts the waiting time.
///
/// # Arguments
///
/// - `delay`: Maximum delay for a device to respond.
/// - `multicast_if`: Multicast network interface to use if provided.
/// - `minissdp_sock`: Minissdpd sock if provided.
/// - `localport`: Local port (source) to use.
/// - `ipv6`: use IPv6.
/// - `ttl`: sets the value of `IP_MULTICAST_TTL` or `IPV6_MULTICAST_HOPS`
/// respectively depending on which protocol is used. Default value is `2`.
///
/// # Errors
///
/// This function might return an error on Out-of-memory or when the socket
/// fails.
pub fn discover(
    delay: Duration,
    multicast_if: Option<&str>,
    minissdp_sock: Option<&str>,
    localport: LocalPort,
    ipv6: bool,
    ttl: u8,
) -> Result<DeviceList, Error> {
    let delay = delay.as_millis() as c_int;
    let multicastif = multicast_if.map(|v| CString::new(v).unwrap());
    let minissdpsock = minissdp_sock.map(|v| CString::new(v).unwrap());
    let localport: c_int = match localport {
        LocalPort::Any => miniupnpc_sys::UPNP_LOCAL_PORT_ANY as c_int,
        LocalPort::Same => miniupnpc_sys::UPNP_LOCAL_PORT_SAME as c_int,
        LocalPort::Other(v) => v as c_int,
    };
    let ipv6: c_int = if ipv6 { 1 } else { 0 };
    let ttl: c_uchar = ttl as c_uchar;

    let mut error: c_int = 0;
    let ptr = unsafe {
        miniupnpc_sys::upnpDiscover(
            delay,
            if let Some(multicastif) = multicastif {
                multicastif.as_c_str().as_ptr()
            } else {
                ptr::null()
            },
            if let Some(minissdpsock) = minissdpsock {
                minissdpsock.as_c_str().as_ptr()
            } else {
                ptr::null()
            },
            localport,
            ipv6,
            ttl,
            (&mut error) as *mut c_int,
        )
    };

    if ptr.is_null() && error != miniupnpc_sys::UPNPDISCOVER_SUCCESS as c_int {
        return Err(Error(error));
    }

    Ok(DeviceList(ptr))
}

/// uPnP discovered devices list
#[derive(Debug)]
pub struct DeviceList(*mut miniupnpc_sys::UPNPDev);

impl DeviceList {
    /// Returns an iterator over the device list
    ///
    /// If the list is empty the iterator will return `None` on
    /// first iteration.
    pub fn iter(&self) -> DeviceListIter<'_> {
        DeviceListIter {
            current: self.0,
            _marker: PhantomData,
        }
    }

    /// Is the device list empty?
    pub fn is_empty(&self) -> bool {
        self.0.is_null()
    }

    /// Get valid IGD from the device list
    pub fn get_valid_igd(&self) -> Option<ValidIgd> {
        unsafe {
            let mut urls: miniupnpc_sys::UPNPUrls =
                MaybeUninit::zeroed().assume_init();
            let mut data: miniupnpc_sys::IGDdatas =
                MaybeUninit::zeroed().assume_init();
            let mut lanaddr: [u8; 64] = [0; 64];

            let ret = miniupnpc_sys::UPNP_GetValidIGD(
                self.0,
                (&mut urls) as *mut miniupnpc_sys::UPNPUrls,
                (&mut data) as *mut miniupnpc_sys::IGDdatas,
                lanaddr.as_mut_ptr() as *mut c_char,
                lanaddr.len() as c_int,
            );

            let (is_igd, is_connected) = match ret {
                // No IGD found
                0 => return None,
                // Valid IGD found
                1 => (true, true),
                // Valid IGD found but not connected
                2 => (true, false),
                // UPnP device found, but was not IGD
                3 => (false, false),
                _ => unreachable!(),
            };

            let lan_address = CStr::from_ptr(lanaddr.as_ptr() as *const c_char)
                .to_string_lossy()
                .into_owned();

            Some(ValidIgd {
                urls: Urls(urls),
                data: if is_igd { Some(IgdData(data)) } else { None },
                lan_address: SocketAddrV4::from_str(lan_address.as_ref())
                    .unwrap_or(SocketAddrV4::new(
                        Ipv4Addr::from_str(lan_address.as_ref()).unwrap(),
                        1900,
                    )),
                connected: is_connected,
            })
        }
    }
}

impl Drop for DeviceList {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe { miniupnpc_sys::freeUPNPDevlist(self.0) }
            self.0 = ptr::null_mut();
        }
    }
}

unsafe impl Send for DeviceList {}

/// Iterator over a [`DeviceList`]
#[derive(Debug)]
pub struct DeviceListIter<'list> {
    current: *mut miniupnpc_sys::UPNPDev,
    _marker: PhantomData<&'list mut miniupnpc_sys::UPNPDev>,
}

impl<'list> Iterator for DeviceListIter<'list> {
    type Item = Device<'list>;

    fn next(&mut self) -> Option<Device<'list>> {
        if self.current.is_null() {
            return None;
        }

        let current = self.current;
        self.current = unsafe { (*current).pNext };
        Some(Device(current, PhantomData))
    }
}

/// Single uPnP device.
pub struct Device<'list>(
    *mut miniupnpc_sys::UPNPDev,
    PhantomData<&'list mut miniupnpc_sys::UPNPDev>,
);

impl<'list> Device<'list> {
    /// Description URL
    pub fn desc_url(&self) -> Cow<'list, str> {
        unsafe {
            let cstr = CStr::from_ptr((*self.0).descURL);
            cstr.to_string_lossy()
        }
    }

    /// Returns the scope ID associated with this device.
    pub fn scope_id(&self) -> u32 {
        let sid = unsafe { (*self.0).scope_id };
        sid as u32
    }
}

unsafe impl<'list> Send for Device<'list> {}

/// Information about a valid IGD
#[derive(Debug)]
pub struct ValidIgd {
    pub urls: Urls,
    pub data: Option<IgdData>,
    pub lan_address: SocketAddrV4,
    pub connected: bool,
}

/// UPnP IGD device URLs
pub struct Urls(miniupnpc_sys::UPNPUrls);

impl Urls {
    /// Control URL of WANIPConnection
    pub fn control_url(&self) -> Cow<'_, str> {
        unsafe { CStr::from_ptr(self.0.controlURL).to_string_lossy() }
    }

    /// URL of the description of WANIPConnection
    pub fn ipcon_desc_url(&self) -> Cow<'_, str> {
        unsafe { CStr::from_ptr(self.0.ipcondescURL).to_string_lossy() }
    }

    /// Control URL of the WANCommonInterfaceConfig
    pub fn control_url_cif(&self) -> Cow<'_, str> {
        unsafe { CStr::from_ptr(self.0.controlURL_CIF).to_string_lossy() }
    }

    /// Control URL of the WANIPv6FirewallControl
    pub fn control_url_6fc(&self) -> Cow<'_, str> {
        unsafe { CStr::from_ptr(self.0.controlURL_6FC).to_string_lossy() }
    }

    /// Root description URL
    pub fn root_desc_url(&self) -> Cow<'_, str> {
        unsafe { CStr::from_ptr(self.0.rootdescURL).to_string_lossy() }
    }
}

impl Drop for Urls {
    fn drop(&mut self) {
        unsafe {
            miniupnpc_sys::FreeUPNPUrls(
                (&mut self.0) as *mut miniupnpc_sys::UPNPUrls,
            );
        }
    }
}

impl fmt::Debug for Urls {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Urls")
            .field("controlURL", &self.control_url())
            .field("ipcondescURL", &self.ipcon_desc_url())
            .field("controlURL_CIF", &self.control_url_cif())
            .field("controlURL_6FC", &self.control_url_6fc())
            .finish()
    }
}

unsafe impl Send for Urls {}
unsafe impl Sync for Urls {}

/// IGD data
pub struct IgdData(miniupnpc_sys::IGDdatas);

impl IgdData {
    // TODO: decipher the meaning of this :-(
    pub fn cur_elt_name(&self) -> Cow<'_, str> {
        unsafe { CStr::from_ptr(self.0.cureltname.as_ptr()).to_string_lossy() }
    }

    pub fn url_base(&self) -> Cow<'_, str> {
        unsafe { CStr::from_ptr(self.0.urlbase.as_ptr()).to_string_lossy() }
    }

    pub fn presentation_url(&self) -> Cow<'_, str> {
        unsafe {
            CStr::from_ptr(self.0.presentationurl.as_ptr()).to_string_lossy()
        }
    }

    pub fn level(&self) -> i32 {
        self.0.level as i32
    }

    pub fn cif(&self) -> IgdService<'_> {
        IgdService(&self.0.CIF)
    }

    pub fn first(&self) -> IgdService<'_> {
        IgdService(&self.0.first)
    }

    pub fn second(&self) -> IgdService<'_> {
        IgdService(&self.0.second)
    }

    pub fn ipv6_fc(&self) -> IgdService<'_> {
        IgdService(&self.0.IPv6FC)
    }

    pub fn tmp(&self) -> IgdService<'_> {
        IgdService(&self.0.tmp)
    }
}

impl fmt::Debug for IgdData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("IgdData")
            .field("cureltname", &self.cur_elt_name())
            .field("urlbase", &self.url_base())
            .field("presentationurl", &self.presentation_url())
            .field("level", &self.level())
            .field("CIF", &self.cif())
            .field("first", &self.first())
            .field("second", &self.second())
            .field("IPv6FC", &self.ipv6_fc())
            .field("tmp", &self.tmp())
            .finish()
    }
}

unsafe impl Send for IgdData {}
unsafe impl Sync for IgdData {}
/// IGD service information
pub struct IgdService<'a>(&'a miniupnpc_sys::IGDdatas_service);

impl<'a> IgdService<'a> {
    pub fn control_url(&self) -> Cow<'a, str> {
        unsafe { CStr::from_ptr(self.0.controlurl.as_ptr()).to_string_lossy() }
    }

    pub fn event_sub_url(&self) -> Cow<'a, str> {
        unsafe { CStr::from_ptr(self.0.eventsuburl.as_ptr()).to_string_lossy() }
    }

    pub fn scpd_url(&self) -> Cow<'a, str> {
        unsafe { CStr::from_ptr(self.0.scpdurl.as_ptr()).to_string_lossy() }
    }

    pub fn service_type(&self) -> Cow<'a, str> {
        unsafe { CStr::from_ptr(self.0.servicetype.as_ptr()).to_string_lossy() }
    }
}

impl<'a> fmt::Debug for IgdService<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("IgdService")
            .field("controlurl", &self.control_url())
            .field("eventsuburl", &self.event_sub_url())
            .field("scpdurl", &self.scpd_url())
            .field("servicetype", &self.service_type())
            .finish()
    }
}

/// # UPnP Commands
pub mod commands {
    use std::ffi::{CStr, CString};
    use std::os::raw::{c_char, c_int, c_uint};
    use std::ptr;

    use std::net::{Ipv4Addr, SocketAddrV4};
    use std::str::FromStr;
    use std::time::Duration;

    use crate::Error;

    #[derive(Debug, Clone, Eq, Copy, PartialEq, Hash)]
    pub enum Protocol {
        Tcp,
        Udp,
    }

    /// Add port mapping
    #[allow(non_snake_case)]
    #[allow(clippy::too_many_arguments)]
    pub fn add_port_mapping<C, S, D>(
        control_url: C,
        service_type: S,
        ext_port: u16,
        in_port: u16,
        in_client: SocketAddrV4,
        desc: D,
        proto: Protocol,
        remote_host: Option<&str>,
        lease_duration: Duration,
    ) -> Result<(), Error>
    where
        C: AsRef<str>,
        S: AsRef<str>,
        D: AsRef<str>,
    {
        let controlURL = CString::new(control_url.as_ref()).unwrap();
        let servicetype = CString::new(service_type.as_ref()).unwrap();
        let extPort = CString::new(ext_port.to_string()).unwrap();
        let inPort = CString::new(in_port.to_string()).unwrap();
        let inClient = CString::new(in_client.to_string()).unwrap();
        let desc = CString::new(desc.as_ref()).unwrap();
        let proto = CString::new(match proto {
            Protocol::Tcp => "TCP",
            Protocol::Udp => "UDP",
        })
        .unwrap();
        let remoteHost = remote_host.map(|v| CString::new(v).unwrap());
        let leaseDuration =
            CString::new(lease_duration.as_secs().to_string()).unwrap();

        unsafe {
            let ret = miniupnpc_sys::UPNP_AddPortMapping(
                controlURL.as_c_str().as_ptr(),
                servicetype.as_c_str().as_ptr(),
                extPort.as_c_str().as_ptr(),
                inPort.as_c_str().as_ptr(),
                inClient.as_c_str().as_ptr(),
                desc.as_c_str().as_ptr(),
                proto.as_c_str().as_ptr(),
                match remoteHost {
                    Some(v) => v.as_c_str().as_ptr(),
                    None => ptr::null(),
                },
                leaseDuration.as_c_str().as_ptr(),
            );

            if ret != miniupnpc_sys::UPNPCOMMAND_SUCCESS as c_int {
                return Err(Error(ret));
            }

            Ok(())
        }
    }

    /// Get connection type information.
    #[allow(non_snake_case)]
    pub fn get_connection_type_info<C, S>(
        control_url: C,
        service_type: S,
    ) -> Result<String, Error>
    where
        C: AsRef<str>,
        S: AsRef<str>,
    {
        let controlURL = CString::new(control_url.as_ref()).unwrap();
        let servicetype = CString::new(service_type.as_ref()).unwrap();
        let mut connectionType: [c_char; 64] = [0; 64];

        unsafe {
            let ret = miniupnpc_sys::UPNP_GetConnectionTypeInfo(
                controlURL.as_c_str().as_ptr(),
                servicetype.as_c_str().as_ptr(),
                connectionType.as_mut_ptr(),
            );

            if ret != miniupnpc_sys::UPNPCOMMAND_SUCCESS as c_int {
                return Err(Error(ret));
            }

            Ok(CStr::from_ptr(connectionType.as_ptr())
                .to_string_lossy()
                .into_owned())
        }
    }

    /// Get External IPv4 address
    ///
    /// This asks for an IGD device for our external IPv4 address if we are
    /// behind a NATed device/router.
    ///
    /// # Arguments
    ///
    /// - `control_url`: IGD control URL.
    /// - `service_type`: IGD service type.
    #[allow(non_snake_case)]
    pub fn get_external_ip_address<C, S>(
        control_url: C,
        service_type: S,
    ) -> Result<Ipv4Addr, Error>
    where
        C: AsRef<str>,
        S: AsRef<str>,
    {
        let controlURL = CString::new(control_url.as_ref()).unwrap();
        let servicetype = CString::new(service_type.as_ref()).unwrap();
        let mut extIpAdd = [0u8; 40];

        let ipaddr = unsafe {
            let ret = miniupnpc_sys::UPNP_GetExternalIPAddress(
                controlURL.as_c_str().as_ptr(),
                servicetype.as_c_str().as_ptr(),
                extIpAdd.as_mut_ptr() as *mut c_char,
            );

            if ret != miniupnpc_sys::UPNPCOMMAND_SUCCESS as c_int {
                return Err(Error(ret));
            }

            CStr::from_ptr(extIpAdd.as_ptr() as *const c_char)
                .to_string_lossy()
                .into_owned()
        };

        Ok(Ipv4Addr::from_str(ipaddr.as_str()).unwrap())
    }

    /// UPnP status information
    #[derive(Debug, Clone, Eq, PartialEq)]
    pub struct StatusInfo {
        pub status: String,
        pub uptime: Duration,
        pub lastconnerror: String,
    }

    /// Get status info
    #[allow(non_snake_case)]
    pub fn get_status_info<C, S>(
        control_url: C,
        service_type: S,
    ) -> Result<StatusInfo, Error>
    where
        C: AsRef<str>,
        S: AsRef<str>,
    {
        let controlURL = CString::new(control_url.as_ref()).unwrap();
        let servicetype = CString::new(service_type.as_ref()).unwrap();
        let mut status: [c_char; 64] = [0; 64];
        let mut uptime: c_uint = 0;
        let mut lastconnerr: [c_char; 64] = [0; 64];

        unsafe {
            let ret = miniupnpc_sys::UPNP_GetStatusInfo(
                controlURL.as_c_str().as_ptr(),
                servicetype.as_c_str().as_ptr(),
                status.as_mut_ptr(),
                (&mut uptime) as *mut c_uint,
                lastconnerr.as_mut_ptr(),
            );

            if ret != miniupnpc_sys::UPNPCOMMAND_SUCCESS as c_int {
                return Err(Error(ret));
            }

            Ok(StatusInfo {
                status: CStr::from_ptr(status.as_ptr())
                    .to_string_lossy()
                    .into_owned(),
                uptime: Duration::from_secs(uptime as u64),
                lastconnerror: CStr::from_ptr(status.as_ptr())
                    .to_string_lossy()
                    .into_owned(),
            })
        }
    }
}
