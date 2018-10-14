use na::Real;
use pipeline::narrow_phase::proximity_detector::{
    BallBallProximityDetector, CompositeShapeShapeProximityDetector,
    PlaneSupportMapProximityDetector, ProximityAlgorithm, ProximityDispatcher,
    SupportMapPlaneProximityDetector, SupportMapSupportMapProximityDetector,
};
use shape::{Ball, Plane, Shape};

/// Proximity dispatcher for shapes defined by `ncollide_entities`.
pub struct DefaultProximityDispatcher {}

impl DefaultProximityDispatcher {
    /// Creates a new basic proximity dispatcher.
    pub fn new() -> DefaultProximityDispatcher {
        DefaultProximityDispatcher {}
    }
}

impl<N: Real> ProximityDispatcher<N> for DefaultProximityDispatcher {
    fn get_proximity_algorithm(&self, a: &Shape<N>, b: &Shape<N>) -> Option<ProximityAlgorithm<N>> {
        let a_is_ball = a.is_shape::<Ball<N>>();
        let b_is_ball = b.is_shape::<Ball<N>>();

        if a_is_ball && b_is_ball {
            Some(Box::new(BallBallProximityDetector::new()))
        } else if a.is_shape::<Plane<N>>() && b.is_support_map() {
            Some(Box::new(PlaneSupportMapProximityDetector::new()))
        } else if b.is_shape::<Plane<N>>() && a.is_support_map() {
            Some(Box::new(SupportMapPlaneProximityDetector::new()))
        } else if a.is_support_map() && b.is_support_map() {
            Some(Box::new(SupportMapSupportMapProximityDetector::new()))
        } else if a.is_composite_shape() {
            Some(Box::new(CompositeShapeShapeProximityDetector::<N>::new(
                false,
            )))
        } else if b.is_composite_shape() {
            Some(Box::new(CompositeShapeShapeProximityDetector::<N>::new(
                true,
            )))
        } else {
            None
        }
    }
}
