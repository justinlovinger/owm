use std::collections::{HashMap, HashSet};
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::thread;

use clap::Parser;
use once_cell::sync::Lazy;
use owm::{LayoutGen, Ratio, Rect, Size, Weight, Weights};
use wayland_client::protocol::wl_seat::WlSeat;
use wayland_client::Connection;
use wayland_client::{
    backend::ObjectId,
    protocol::{
        wl_output::{self, WlOutput},
        wl_registry::{self, WlRegistry},
    },
    Dispatch, Proxy,
};

use crate::protocol::{
    river_layout_manager_v3::RiverLayoutManagerV3,
    river_layout_v3::{self, RiverLayoutV3},
    zriver_command_callback_v1::ZriverCommandCallbackV1,
    zriver_control_v1::ZriverControlV1,
};

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[arg(long, value_name = "UINT", default_value_t = 320)]
    min_width: usize,

    #[arg(long, value_name = "UINT", default_value_t = 180)]
    min_height: usize,

    #[arg(long, value_name = "UINT", value_parser = usize_option_parser, default_value = "1920")]
    max_width: std::option::Option<usize>,

    #[arg(long, value_name = "UINT", value_parser = usize_option_parser, default_value = "")]
    max_height: std::option::Option<usize>,

    /// Importance of "minimize gaps" objective.
    #[arg(long, value_name = "WEIGHT", default_value_t = Weight::new(3.0).unwrap())]
    gaps_weight: Weight,

    /// Importance of "minimize overlap" objective.
    #[arg(long, value_name = "WEIGHT", default_value_t = Weight::new(6.0).unwrap())]
    overlap_weight: Weight,

    /// Desired area ratios between each window and the next.
    ///
    /// Values are comma-separated.
    /// Last value is repeated for further pairs.
    #[arg(
        long,
        value_name = "RATIOS",
        value_delimiter = ',',
        default_value = "3,2,1"
    )]
    area_ratios: Vec<Ratio>,

    /// Importance of "maintain area ratio" objective.
    #[arg(long, value_name = "WEIGHT", default_value_t = Weight::new(1.5).unwrap())]
    area_ratio_weight: Weight,

    /// Importance of "place adjacent close" objective.
    #[arg(long, value_name = "WEIGHT", default_value_t = Weight::new(0.5).unwrap())]
    adjacent_close_weight: Weight,

    /// Importance of "place in reading order" objective.
    #[arg(long, value_name = "WEIGHT", default_value_t = Weight::new(0.5).unwrap())]
    reading_order_weight: Weight,

    /// Importance of "center main" objective.
    #[arg(long, value_name = "WEIGHT", default_value_t = Weight::new(1.5).unwrap())]
    center_main_weight: Weight,
}

fn usize_option_parser(s: &str) -> Result<Option<usize>, <usize as FromStr>::Err> {
    option_parser(s)
}

fn option_parser<T>(s: &str) -> Result<Option<T>, <T as FromStr>::Err>
where
    T: FromStr,
{
    if s.is_empty() {
        Ok(None)
    } else {
        s.parse().map(Some)
    }
}

fn main() {
    let args = Args::parse();

    let mut layout_manager = LayoutManager::new(LayoutGen::new(
        args.min_width,
        args.min_height,
        args.max_width,
        args.max_height,
        Weights {
            gaps_weight: args.gaps_weight,
            overlap_weight: args.overlap_weight,
            area_ratios_weight: args.area_ratio_weight,
            adjacent_close_weight: args.adjacent_close_weight,
            reading_order_weight: args.reading_order_weight,
            center_main_weight: args.center_main_weight,
        },
        args.area_ratios,
    ));

    let conn = Connection::connect_to_env().unwrap();
    let mut event_queue = conn.new_event_queue();
    // `get_registry` has necessary side-effects.
    let _registry = conn.display().get_registry(&event_queue.handle(), ());
    event_queue.roundtrip(&mut layout_manager).unwrap();
    loop {
        event_queue.blocking_dispatch(&mut layout_manager).unwrap();
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct OutputId(ObjectId);

impl OutputId {
    pub fn new(output: &WlOutput) -> OutputId {
        OutputId(output.id())
    }
}

pub struct LayoutManager {
    gen: LayoutGen,
    // These will be initialized
    // by Wayland events.
    seat: Option<Arc<WlSeat>>,
    manager: Option<RiverLayoutManagerV3>,
    control: Option<Arc<Mutex<ZriverControlV1>>>,
}

impl LayoutManager {
    pub fn new(gen: LayoutGen) -> Self {
        Self {
            gen,
            seat: None,
            manager: None,
            control: None,
        }
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
                "wl_seat" => {
                    state.seat = Some(Arc::new(registry.bind::<WlSeat, _, Self>(
                        name,
                        version,
                        qhandle,
                        (),
                    )));
                }
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
                "zriver_control_v1" => {
                    state.control = Some(Arc::new(Mutex::new(
                        registry.bind::<ZriverControlV1, _, Self>(name, version, qhandle, ()),
                    )));
                }
                _ => {}
            }
        }
    }
}

impl Dispatch<WlSeat, ()> for LayoutManager {
    fn event(
        _: &mut Self,
        _: &WlSeat,
        _: <WlSeat as wayland_client::Proxy>::Event,
        _: &(),
        _: &wayland_client::Connection,
        _: &wayland_client::QueueHandle<Self>,
    ) {
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
        state: &mut Self,
        proxy: &RiverLayoutV3,
        event: <RiverLayoutV3 as wayland_client::Proxy>::Event,
        _output: &OutputId,
        conn: &wayland_client::Connection,
        qhandle: &wayland_client::QueueHandle<Self>,
    ) {
        match event {
            river_layout_v3::Event::LayoutDemand {
                view_count,
                usable_width,
                usable_height,
                tags: _,
                serial,
            } => {
                type Key = (Size, usize);
                static CACHE: Lazy<Mutex<HashMap<Key, Vec<Rect>>>> =
                    Lazy::new(|| Mutex::new(HashMap::new()));
                static STARTED: Lazy<Mutex<HashSet<Key>>> =
                    Lazy::new(|| Mutex::new(HashSet::new()));

                let container = Size::new(usable_width as usize, usable_height as usize);
                let view_count = view_count as usize;
                let key = (container, view_count);

                match CACHE.lock().unwrap().get(&key) {
                    Some(layout) => {
                        for rect in layout {
                            proxy.push_view_dimensions(
                                rect.x() as i32,
                                rect.y() as i32,
                                rect.width() as u32,
                                rect.height() as u32,
                                serial,
                            );
                        }
                        proxy.commit("owm".to_owned(), serial);
                    }
                    None => {
                        if STARTED.lock().unwrap().insert(key) {
                            let gen = state.gen.clone();
                            let control = Arc::clone(
                                state
                                    .control
                                    .as_ref()
                                    .expect("River control should be initialized"),
                            );
                            let seat = Arc::clone(
                                state.seat.as_ref().expect("seat should be initialized"),
                            );
                            let qhandle = qhandle.clone();
                            let conn = conn.clone();
                            thread::spawn(move || {
                                let layout = gen.layout(container, view_count);
                                CACHE.lock().unwrap().insert(key, layout);

                                // River will send a new layout demand
                                // if it receives a layout command.
                                let control = control.lock().unwrap();
                                control.add_argument("send-layout-cmd".to_owned());
                                control.add_argument("owm".to_owned());
                                control.add_argument("retry-layout".to_owned());
                                control.run_command(&seat, &qhandle, ());
                                let _ = conn.flush();
                            });
                        }
                    }
                }
            }
            river_layout_v3::Event::NamespaceInUse => {
                panic!("namespace in use: layout program may already be running");
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

impl Dispatch<ZriverControlV1, ()> for LayoutManager {
    fn event(
        _: &mut Self,
        _: &ZriverControlV1,
        _: <ZriverControlV1 as wayland_client::Proxy>::Event,
        _: &(),
        _: &wayland_client::Connection,
        _: &wayland_client::QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<ZriverCommandCallbackV1, ()> for LayoutManager {
    fn event(
        _: &mut Self,
        _: &ZriverCommandCallbackV1,
        _: <ZriverCommandCallbackV1 as wayland_client::Proxy>::Event,
        _: &(),
        _: &wayland_client::Connection,
        _: &wayland_client::QueueHandle<Self>,
    ) {
    }
}

mod protocol {
    // See <https://docs.rs/wayland-scanner/latest/wayland_scanner/#example-usage>.

    #![allow(non_upper_case_globals)]

    #[allow(clippy::single_component_path_imports)] // Used by macros.
    use wayland_client;
    use wayland_client::protocol::*;

    pub mod __layout_interfaces {
        use wayland_client::backend as wayland_backend;
        use wayland_client::protocol::__interfaces::*;

        wayland_scanner::generate_interfaces!("./protocols/river-layout-v3.xml");
    }
    use self::__layout_interfaces::*;

    pub mod __control_interfaces {
        use wayland_client::backend as wayland_backend;
        use wayland_client::protocol::__interfaces::*;

        wayland_scanner::generate_interfaces!("./protocols/river-control-unstable-v1.xml");
    }
    use self::__control_interfaces::*;

    wayland_scanner::generate_client_code!("./protocols/river-layout-v3.xml");
    wayland_scanner::generate_client_code!("./protocols/river-control-unstable-v1.xml");
}
