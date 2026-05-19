//! Profiling related functionality

use core::net::IpAddr;

pub use profiling::*;

#[cfg(all(feature = "profiling", not(target_arch = "wasm32")))]
mod internal {

    use core::net::{IpAddr, Ipv4Addr};
    use std::sync::Mutex;

    use puffin_http::Server;

    static CURRENT_HTTP_SERVER: Mutex<Option<Server>> = Mutex::new(None);

    pub(super) fn set_profiling_state() {
        let should_be_enabled = CURRENT_HTTP_SERVER.lock().unwrap().is_some();

        profiling::puffin::set_scopes_on(should_be_enabled);
    }

    pub(super) fn start_http_server_impl(addr: Option<IpAddr>, port: Option<u16>) {
        let mut global_server = CURRENT_HTTP_SERVER.lock().unwrap();

        if global_server.is_some() {
            return;
        }

        let full_addr = format!(
            "{}:{}",
            addr.unwrap_or(Ipv4Addr::UNSPECIFIED.into()),
            port.unwrap_or(puffin_http::DEFAULT_PORT)
        );

        log::info!("Starting profiling HTTP server on address: {full_addr}");

        let server = match puffin_http::Server::new(&full_addr) {
            Ok(s) => s,
            Err(e) => {
                log::error!("Failed to start puffin HTTP server: {}", e);
                return;
            }
        };

        *global_server = Some(server);
    }

    pub(super) fn stop_http_server_impl() {
        let mut global_server = CURRENT_HTTP_SERVER.lock().unwrap();

        let srv = global_server.take();

        if srv.is_some() {
            log::info!("Stopping profiling HTTP server");
        }
    }
}

/// Starts the profiler HTTP server at the given address and port
#[allow(unused_variables, reason = "Does nothing in builds without profiling")]
pub fn start_http_server(addr: Option<IpAddr>, port: Option<u16>) {
    #[cfg(all(feature = "profiling", not(target_arch = "wasm32")))]
    {
        internal::start_http_server_impl(addr, port);
        internal::set_profiling_state();
    }
}

/// Stops the current profiler HTTP server
#[allow(unused_variables, reason = "Does nothing in builds without profiling")]
pub fn stop_http_server() {
    #[cfg(all(feature = "profiling", not(target_arch = "wasm32")))]
    {
        internal::stop_http_server_impl();
        internal::set_profiling_state();
    }
}
