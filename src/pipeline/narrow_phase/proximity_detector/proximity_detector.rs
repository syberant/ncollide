use crate::math::Isometry;
use na::RealField;
use crate::query::Proximity;
use crate::shape::Shape;
use std::any::Any;

/// Trait implemented by algorithms that determine if two objects are in close proximity.
pub trait ProximityDetector<N: RealField>: Any + Send + Sync {
    /// Runs the proximity detection on two objects. It is assumed that the same proximity detector
    /// (the same structure) is always used with the same pair of object.
    fn update(
        &mut self,
        dispatcher: &dyn ProximityDispatcher<N>,
        ma: &Isometry<N>,
        a: &dyn Shape<N>,
        mb: &Isometry<N>,
        b: &dyn Shape<N>,
        margin: N,
    ) -> bool;

    /// The number of collision detected during the last update.
    fn proximity(&self) -> Proximity;
}

pub type ProximityAlgorithm<N> = Box<dyn ProximityDetector<N>>;

pub trait ProximityDispatcher<N>: Any + Send + Sync {
    /// Allocate a collision algorithm corresponding to the given pair of shapes.
    fn get_proximity_algorithm(&self, a: &dyn Shape<N>, b: &dyn Shape<N>) -> Option<ProximityAlgorithm<N>>;
}
