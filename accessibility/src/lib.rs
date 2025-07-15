mod a11y_tree;
pub mod id;
mod node;
mod traits;

pub use a11y_tree::*;
pub use accesskit;
use accesskit::{ActionRequest, TreeUpdate};
pub use id::*;
pub use node::*;
pub use traits::*;

#[cfg(feature = "accesskit_macos")]
pub use accesskit_macos;
#[cfg(feature = "accesskit_unix")]
pub use accesskit_unix;
#[cfg(feature = "accesskit_windows")]
pub use accesskit_windows;

#[derive(Debug)]
pub struct Event<WindowId: std::fmt::Debug = ()> {
    pub window_id: WindowId,
    pub window_event: WindowEvent,
}

pub struct PlatformAdapter;

impl PlatformAdapter {
    pub fn new<Window, EventLoopProxy>(
        _window: &Window,
        _: impl FnOnce() -> TreeUpdate,
        _: EventLoopProxy,
    ) -> Self {
        PlatformAdapter
    }
}

impl Adapter for PlatformAdapter {
    type Window = ();

    fn update_if_active(&mut self, _updater: impl FnOnce() -> TreeUpdate) {
        todo!()
    }

    fn process_event(&mut self, _window: &Self::Window, _event: &WindowEvent) {
        todo!()
    }

    fn update(&mut self, _update: TreeUpdate) {
        todo!()
    }
}

#[derive(Debug)]
pub enum WindowEvent {
    InitialTreeRequested,
    ActionRequested(ActionRequest),
    AccessibilityDeactivated,
}

pub trait Adapter {
    type Window;
    fn update_if_active(&mut self, updater: impl FnOnce() -> TreeUpdate);
    fn process_event(&mut self, _window: &Self::Window, event: &WindowEvent);
    fn update(&mut self, _update: TreeUpdate);
}
