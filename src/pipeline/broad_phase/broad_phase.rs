use na::RealField;
use std::any::Any;

use crate::math::Point;
use crate::query::Ray;

#[cfg_attr(feature = "serde-serialize", derive(serde::Serialize, serde::Deserialize))]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ProxyHandle(pub usize);

impl ProxyHandle {
    #[inline]
    pub fn invalid() -> Self {
        ProxyHandle(usize::max_value())
    }

    #[inline]
    pub fn is_invalid(&self) -> bool {
        self.0 == usize::max_value()
    }

    #[inline]
    pub fn uid(&self) -> usize {
        self.0
    }
}

/// Proximity handling for BroadPhase updates.
pub trait BroadPhaseInterferenceHandler<T> {
    /// A pre-filter that may cheaply discard objects before checking for bounding volume
    /// interference.
    fn is_interference_allowed(&mut self, data1: &T, data2: &T) -> bool;

    /// Handle a starting interference.
    fn interference_started(&mut self, data1: &T, data2: &T);

    /// Handle a stopping interference.
    fn interference_stopped(&mut self, data1: &T, data2: &T);
}

/// Trait all broad phase must implement.
pub trait BroadPhase<N: RealField, BV, T>: Any + Sync + Send {
    /// Tells the broad phase to add a bounding-volume at the next update.
    fn create_proxy(&mut self, bv: BV, data: T) -> ProxyHandle;

    /// Tells the broad phase to remove the given set of handles.
    fn remove(&mut self, handles: &[ProxyHandle], removal_handler: &mut dyn FnMut(&T, &T));

    /// Sets the next bounding volume to be used during the update of this broad phase.
    fn deferred_set_bounding_volume(&mut self, handle: ProxyHandle, bv: BV);

    /// Forces the broad-phase to recompute and re-report all the proximities with the given object.
    fn deferred_recompute_all_proximities_with(&mut self, handle: ProxyHandle);

    /// Forces the broad-phase to recompute and re-report all the proximities.
    fn deferred_recompute_all_proximities(&mut self);

    /// Updates the object additions, removals, and interferences detection.
    fn update(&mut self, handler: &mut dyn BroadPhaseInterferenceHandler<T>);

    /*
     * FIXME: the following are not flexible enough.
     */
    // XXX: return iterators when associated types work.
    /// Collects every object which might intersect a given bounding volume.
    fn interferences_with_bounding_volume<'a>(&'a self, bv: &BV, out: &mut Vec<&'a T>);

    /// Collects every object which might intersect a given ray.
    fn interferences_with_ray<'a>(&'a self, ray: &Ray<N>, out: &mut Vec<&'a T>);

    /// Collects every object which might contain a given point.
    fn interferences_with_point<'a>(&'a self, point: &Point<N>, out: &mut Vec<&'a T>);
}
