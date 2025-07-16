//! Widget and Window IDs.

use std::hash::Hash;
use std::sync::atomic::{self, AtomicU64};
use std::{borrow, num::NonZeroU128};

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub enum A11yId {
    Window(WindowId),
    Widget(Id),
}
// impl A11yId {
//     pub fn new_widget() -> Self {
//         Self::Widget(Id::unique())
//     }

//     pub fn new_window() -> Self {
//         Self::Window(window_node_id())
//     }
// }

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub struct WindowId(NonZeroU128);

impl From<WindowId> for A11yId {
    fn from(id: WindowId) -> Self {
        Self::Window(id)
    }
}

impl From<NonZeroU128> for WindowId {
    fn from(value: NonZeroU128) -> Self {
        Self(value)
    }
}

impl From<u64> for WindowId {
    fn from(value: u64) -> Self {
        let value = value as u128;
        Self(NonZeroU128::new(value + u64::MAX as u128).unwrap())
    }
}

impl From<WindowId> for u64 {
    fn from(value: WindowId) -> Self {
        (value.0.get() - u64::MAX as u128) as u64
    }
}

impl From<Id> for A11yId {
    fn from(id: Id) -> Self {
        assert!(!matches!(id.0, Internal::Set(_)));
        Self::Widget(id)
    }
}

impl TryFrom<Id> for u64 {
    type Error = ();

    fn try_from(value: Id) -> Result<Self, Self::Error> {
        match value.0 {
            Internal::Unique(id) => Ok(id),
            Internal::Custom(id, _) => Ok(id),
            Internal::Set(_) => Err(()),
        }
    }
}

impl From<accesskit::NodeId> for A11yId {
    fn from(value: accesskit::NodeId) -> Self {
        Self::Widget(Id::from(value.0))
    }
}

impl From<A11yId> for accesskit::NodeId {
    fn from(value: A11yId) -> Self {
        match value {
            A11yId::Window(window_id) => Self(window_id.into()),
            A11yId::Widget(id) => Self(id.try_into().unwrap()),
        }
    }
}

static NEXT_ID: AtomicU64 = AtomicU64::new(1);
static NEXT_WINDOW_ID: AtomicU64 = AtomicU64::new(1);

/// The identifier of a generic widget.
#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub struct Id(pub Internal);

impl Id {
    /// Creates a custom [`Id`].
    pub fn new(id: impl Into<borrow::Cow<'static, str>>) -> Self {
        Self(Internal::Custom(Self::next(), id.into()))
    }

    /// resets the id counter
    pub fn reset() {
        NEXT_ID.store(1, atomic::Ordering::Relaxed);
    }

    fn next() -> u64 {
        NEXT_ID.fetch_add(1, atomic::Ordering::Relaxed)
    }

    /// Creates a unique [`Id`].
    ///
    /// This function produces a different [`Id`] every time it is called.
    pub fn unique() -> Self {
        let id = Self::next();

        Self(Internal::Unique(id))
    }
}

// Not meant to be used directly
impl From<u64> for Id {
    fn from(value: u64) -> Self {
        Self(Internal::Unique(value))
    }
}

// Not meant to be used directly
impl From<Id> for NonZeroU128 {
    fn from(val: Id) -> Self {
        match &val.0 {
            Internal::Unique(id) => NonZeroU128::try_from(*id as u128).unwrap(),
            Internal::Custom(id, _) => {
                NonZeroU128::try_from(*id as u128).unwrap()
            }
            // this is a set id, which is not a valid id and will not ever be converted to a NonZeroU128
            // so we panic
            Internal::Set(_) => {
                panic!("Cannot convert a set id to a NonZeroU128")
            }
        }
    }
}

impl ToString for Id {
    fn to_string(&self) -> String {
        match &self.0 {
            Internal::Unique(_) => "Undefined".to_string(),
            Internal::Custom(_, id) => id.to_string(),
            Internal::Set(_) => "Set".to_string(),
        }
    }
}

// XXX WIndow IDs are made unique by adding u64::MAX to them
/// get window node id that won't conflict with other node ids for the duration of the program
pub fn window_node_id() -> A11yId {
    let id: WindowId = NEXT_WINDOW_ID
        .fetch_add(1, atomic::Ordering::Relaxed)
        .into();
    id.into()
}

// TODO refactor to make panic impossible?
#[derive(Debug, Clone, Eq)]
/// Internal representation of an [`Id`].
pub enum Internal {
    /// a unique id
    Unique(u64),
    /// a custom id, which is equal to any [`Id`] with a matching number or string
    Custom(u64, borrow::Cow<'static, str>),
    /// XXX Do not use this as an id for an accessibility node, it will panic!
    /// XXX Only meant to be used for widgets that have multiple accessibility nodes, each with a
    /// unique or custom id
    /// an Id Set, which is equal to any [`Id`] with a matching number or string
    Set(Vec<Self>),
}

impl PartialEq for Internal {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Unique(l0), Self::Unique(r0)) => l0 == r0,
            (Self::Custom(l0, l1), Self::Custom(r0, r1)) => {
                l0 == r0 || l1 == r1
            }
            // allow custom ids to be equal to unique ids
            (Self::Unique(l0), Self::Custom(r0, _))
            | (Self::Custom(l0, _), Self::Unique(r0)) => l0 == r0,
            (Self::Set(l0), Self::Set(r0)) => l0 == r0,
            // allow set ids to just be equal to any of their members
            (Self::Set(l0), r) | (r, Self::Set(l0)) => {
                l0.iter().any(|l| l == r)
            }
        }
    }
}

impl Hash for Internal {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Self::Unique(id) => id.hash(state),
            Self::Custom(name, _) => name.hash(state),
            Self::Set(ids) => ids.hash(state),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use accesskit::NodeId;

    #[test]
    fn unique_generates_different_ids() {
        let a = Id::unique();
        let b = Id::unique();

        assert_ne!(a, b);
    }

    #[test]
    fn widget_id_transformations() {
        for id in [1, 2, u64::MAX] {
            let original = A11yId::Widget(id.into());
            let node_id: NodeId = original.clone().into();
            let transformed = node_id.into();
            assert_eq!(original, transformed, "Incorrect for {}", id);
        }
    }
}
