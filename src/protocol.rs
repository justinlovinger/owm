// See <https://docs.rs/wayland-scanner/latest/wayland_scanner/#example-usage>.

#![allow(non_upper_case_globals)]

#[allow(clippy::single_component_path_imports)] // Used by macros.
use wayland_client;
use wayland_client::protocol::*;

pub mod __interfaces {
    use wayland_client::backend as wayland_backend;
    use wayland_client::protocol::__interfaces::*;

    wayland_scanner::generate_interfaces!("./protocols/river-layout-v3.xml");
}
use self::__interfaces::*;

wayland_scanner::generate_client_code!("./protocols/river-layout-v3.xml");
