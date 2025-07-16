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

// Always re-export accesskit_winit since it's always available
pub use accesskit_winit;

// Re-export types from accesskit_winit
pub use accesskit_winit::{Adapter as PlatformAdapter, Event, WindowEvent};

// Keep backward compatibility with old Event type
#[derive(Debug)]
pub struct LegacyEvent<WindowId: std::fmt::Debug = ()> {
    pub window_id: WindowId,
    pub window_event: LegacyWindowEvent,
}

#[derive(Debug)]
pub enum LegacyWindowEvent {
    InitialTreeRequested,
    ActionRequested(ActionRequest),
    AccessibilityDeactivated,
}
