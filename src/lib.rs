mod protocol;

use protocol::river_layout_v3;
use wayland_client::{
    backend::ObjectId,
    protocol::{
        wl_output::{self, WlOutput},
        wl_registry::{self, WlRegistry},
    },
    Dispatch, Proxy,
};

use crate::protocol::{
    river_layout_manager_v3::RiverLayoutManagerV3, river_layout_v3::RiverLayoutV3,
};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct OutputId(ObjectId);

impl OutputId {
    pub fn new(output: &WlOutput) -> OutputId {
        OutputId(output.id())
    }
}

pub struct LayoutManager {
    // This will be initialized
    // when River makes a connection.
    manager: Option<RiverLayoutManagerV3>,
}

impl LayoutManager {
    pub fn new() -> Self {
        Self { manager: None }
    }
}

impl Default for LayoutManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Dispatch<WlRegistry, ()> for LayoutManager {
    fn event(
        state: &mut Self,
        registry: &WlRegistry,
        event: <WlRegistry as Proxy>::Event,
        _: &(),
        _: &wayland_client::Connection,
        qhandle: &wayland_client::QueueHandle<Self>,
    ) {
        if let wl_registry::Event::Global {
            name,
            interface,
            version,
        } = event
        {
            match interface.as_str() {
                "wl_output" => {
                    registry.bind::<WlOutput, _, Self>(name, version, qhandle, ());
                }
                "river_layout_manager_v3" => {
                    state.manager = Some(registry.bind::<RiverLayoutManagerV3, _, Self>(
                        name,
                        version,
                        qhandle,
                        (),
                    ));
                }
                _ => {}
            }
        }
    }
}

impl Dispatch<WlOutput, ()> for LayoutManager {
    fn event(
        state: &mut Self,
        output: &WlOutput,
        event: <WlOutput as Proxy>::Event,
        _: &(),
        _: &wayland_client::Connection,
        qhandle: &wayland_client::QueueHandle<Self>,
    ) {
        if let wl_output::Event::Name { name: _ } = event {
            // `get_layout` has necessary side-effects.
            state
                .manager
                .as_ref()
                .expect("compositor should support `river_layout_v3`")
                .get_layout(output, String::from("owm"), qhandle, OutputId::new(output));
        }
    }
}

impl Dispatch<RiverLayoutV3, OutputId> for LayoutManager {
    fn event(
        _state: &mut Self,
        proxy: &RiverLayoutV3,
        event: <RiverLayoutV3 as wayland_client::Proxy>::Event,
        _output: &OutputId,
        _: &wayland_client::Connection,
        _: &wayland_client::QueueHandle<Self>,
    ) {
        match event {
            river_layout_v3::Event::LayoutDemand {
                view_count,
                usable_width,
                usable_height,
                tags: _,
                serial,
            } => {
                // This is a placeholder:
                for x in 0..view_count {
                    proxy.push_view_dimensions(
                        x as i32 * 100,
                        x as i32 * 100,
                        usable_width / (view_count + 1),
                        usable_height / (view_count + 1),
                        serial,
                    );
                }
                proxy.commit("owm".to_owned(), serial);
            }
            river_layout_v3::Event::NamespaceInUse => {
                panic!("namespace in use: layout program already running");
            }
            _ => {}
        }
    }
}

impl Dispatch<RiverLayoutManagerV3, ()> for LayoutManager {
    fn event(
        _: &mut Self,
        _: &RiverLayoutManagerV3,
        _: <RiverLayoutManagerV3 as wayland_client::Proxy>::Event,
        _: &(),
        _: &wayland_client::Connection,
        _: &wayland_client::QueueHandle<Self>,
    ) {
    }
}
