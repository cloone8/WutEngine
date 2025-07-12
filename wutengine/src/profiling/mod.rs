//! Profiling related functionality

use core::net::IpAddr;

pub use profiling::*;

#[cfg(feature = "profiling")]
mod internal;

/// Starts the profiler HTTP server at the given address and port
#[allow(unused_variables)]
pub fn start_http_server(addr: Option<IpAddr>, port: Option<u16>) {
    #[cfg(feature = "profiling")]
    {
        internal::start_http_server_impl(addr, port);
        internal::set_profiling_state();
    }
}

/// Stops the current profiler HTTP server
#[allow(unused_variables)]
pub fn stop_http_server() {
    #[cfg(feature = "profiling")]
    {
        internal::stop_http_server_impl();
        internal::set_profiling_state();
    }
}
