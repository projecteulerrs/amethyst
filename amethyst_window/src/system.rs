use crate::{config::DisplayConfig, resources::ScreenDimensions};
use amethyst_config::Config;
use amethyst_core::{
    ecs::{ReadExpect, Resources, RunNow, System, SystemData, Write, WriteExpect},
    shrev::EventChannel,
};
use std::path::Path;
use winit::{Event, EventsLoop, Window};

/// System for opening and managing the window.
#[derive(Debug)]
pub struct WindowSystem {
    window: Option<Window>,
}

impl WindowSystem {
    /// Builds and spawns a new `Window`, using the provided `DisplayConfig` and `EventsLoop` as
    /// sources. Returns a new `WindowSystem`
    pub fn from_config_path(events_loop: &EventsLoop, path: impl AsRef<Path>) -> Self {
        Self::from_config(events_loop, DisplayConfig::load(path.as_ref()))
    }

    /// Builds and spawns a new `Window`, using the provided `DisplayConfig` and `EventsLoop` as
    /// sources. Returns a new `WindowSystem`
    pub fn from_config(events_loop: &EventsLoop, config: DisplayConfig) -> Self {
        let window = config
            .into_window_builder(events_loop)
            .build(events_loop)
            .unwrap();
        Self::new(window)
    }

    /// Create a new `WindowSystem` wrapping the provided `Window`
    pub fn new(window: Window) -> Self {
        Self {
            window: Some(window),
        }
    }

    fn manage_dimensions(&mut self, mut screen_dimensions: &mut ScreenDimensions, window: &Window) {
        let width = screen_dimensions.w;
        let height = screen_dimensions.h;

        // Send resource size changes to the window
        if screen_dimensions.dirty {
            window.set_inner_size((width, height).into());
            screen_dimensions.dirty = false;
        }

        let hidpi = window.get_hidpi_factor();

        if let Some(size) = window.get_inner_size() {
            let (window_width, window_height): (f64, f64) = size.to_physical(hidpi).into();

            // Send window size changes to the resource
            if (window_width, window_height) != (width, height) {
                screen_dimensions.update(window_width, window_height);

                // We don't need to send the updated size of the window back to the window itself,
                // so set dirty to false.
                screen_dimensions.dirty = false;
            }
        }
        screen_dimensions.update_hidpi_factor(hidpi);
    }
}

impl<'a> System<'a> for WindowSystem {
    type SystemData = (WriteExpect<'a, ScreenDimensions>, ReadExpect<'a, Window>);

    fn run(&mut self, (mut screen_dimensions, window): Self::SystemData) {
        #[cfg(feature = "profiler")]
        profile_scope!("window_system");

        self.manage_dimensions(&mut screen_dimensions, &window);
    }
    fn setup(&mut self, res: &mut Resources) {
        if let Some(window) = self.window.take() {
            let (width, height) = window
                .get_inner_size()
                .expect("Window closed during initialization!")
                .into();
            let hidpi = window.get_hidpi_factor();
            res.insert(ScreenDimensions::new(width, height, hidpi));
            res.insert(window);
        }
    }
}

/// System that polls the window events and pushes them to appropriate event channels.
///
/// This system must be active for any `GameState` to receive
/// any `StateEvent::Window` event into it's `handle_event` method.
#[derive(Debug)]
pub struct EventsLoopSystem {
    events_loop: EventsLoop,
    events: Vec<Event>,
}

impl EventsLoopSystem {
    /// Creates a new `EventsLoopSystem` using the provided `EventsLoop`
    pub fn new(events_loop: EventsLoop) -> Self {
        Self {
            events_loop,
            events: Vec::with_capacity(128),
        }
    }
}

impl<'a> RunNow<'a> for EventsLoopSystem {
    fn run_now(&mut self, res: &'a Resources) {
        let mut event_handler = <Write<'a, EventChannel<Event>>>::fetch(res);

        let events = &mut self.events;
        self.events_loop.poll_events(|event| {
            events.push(event);
        });
        event_handler.drain_vec_write(events);
    }

    fn setup(&mut self, res: &mut Resources) {
        <Write<'a, EventChannel<Event>>>::setup(res);
    }
}
