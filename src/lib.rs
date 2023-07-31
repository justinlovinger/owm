mod binary;
mod encoding;
mod objective;
mod protocol;

use encoding::Decoder;
use ndarray::Axis;
use objective::{evaluate, Size};
use optimal::{optimizer::derivative_free::pbil::*, prelude::*};
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
                // If this takes more than a few milliseconds,
                // River will ignore the result,
                // see <https://github.com/riverwm/river/issues/867>.
                let decoder = Decoder::new(16, usable_width as usize, usable_height as usize);
                let usable_size = Size {
                    width: decoder.width(),
                    height: decoder.height(),
                };
                let windows = decoder.decode1(
                    UntilConvergedConfig::default()
                        .argmin(&mut Config::start_default_for(
                            decoder.bits_per_window() * view_count as usize,
                            |points| {
                                decoder.decode(points).map_axis(Axis(1), |windows| {
                                    evaluate(usable_size, windows.as_slice().unwrap())
                                })
                            },
                        ))
                        .view(),
                );
                for window in windows {
                    proxy.push_view_dimensions(
                        window.pos.x as i32,
                        window.pos.y as i32,
                        window.size.width as u32,
                        window.size.height as u32,
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
