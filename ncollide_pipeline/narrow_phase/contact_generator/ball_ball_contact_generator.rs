use std::marker::PhantomData;

use alga::linear::Translation;
use math::{Isometry, Point};
use utils::IdAllocator;
use geometry::bounding_volume::PolyhedralCone;
use geometry::shape::{Ball, FeatureId, Shape};
use geometry::query::{ContactKinematic, ContactManifold, ContactPrediction};
use geometry::query::contacts_internal;
use narrow_phase::{ContactDispatcher, ContactGenerator};

/// Collision detector between two balls.
pub struct BallBallContactGenerator<P: Point, M> {
    manifold: ContactManifold<P>,
    mat_type: PhantomData<M>, // FIXME: can we avoid this?
}

impl<P: Point, M> Clone for BallBallContactGenerator<P, M> {
    fn clone(&self) -> BallBallContactGenerator<P, M> {
        BallBallContactGenerator {
            manifold: self.manifold.clone(),
            mat_type: PhantomData,
        }
    }
}

impl<P: Point, M> BallBallContactGenerator<P, M> {
    /// Creates a new persistent collision detector between two balls.
    #[inline]
    pub fn new() -> BallBallContactGenerator<P, M> {
        BallBallContactGenerator {
            manifold: ContactManifold::new(),
            mat_type: PhantomData,
        }
    }
}

impl<P: Point, M: Isometry<P>> ContactGenerator<P, M> for BallBallContactGenerator<P, M> {
    fn update(
        &mut self,
        _: &ContactDispatcher<P, M>,
        ma: &M,
        a: &Shape<P, M>,
        mb: &M,
        b: &Shape<P, M>,
        prediction: &ContactPrediction<P::Real>,
        id_alloc: &mut IdAllocator,
    ) -> bool {
        if let (Some(a), Some(b)) = (a.as_shape::<Ball<P::Real>>(), b.as_shape::<Ball<P::Real>>()) {
            self.manifold.save_cache_and_clear(id_alloc);
            let center_a = P::from_coordinates(ma.translation().to_vector());
            let center_b = P::from_coordinates(mb.translation().to_vector());
            if let Some(contact) =
                contacts_internal::ball_against_ball(&center_a, a, &center_b, b, prediction.linear)
            {
                let normals1 = PolyhedralCone::<P>::from_slice(&[contact.normal]);
                let normals2 = PolyhedralCone::<P>::from_slice(&[-contact.normal]);
                let mut kinematic = ContactKinematic::new();
                kinematic.set_point1(FeatureId::face(0, 0), P::origin(), PolyhedralCone::new());
                kinematic.set_point2(FeatureId::face(0, 0), P::origin(), PolyhedralCone::new());
                kinematic.set_dilation1(a.radius());
                kinematic.set_dilation2(b.radius());

                let _ = self.manifold.push(contact, kinematic, id_alloc);
            }

            true
        } else {
            false
        }
    }

    #[inline]
    fn num_contacts(&self) -> usize {
        self.manifold.len()
    }

    #[inline]
    fn contacts<'a: 'b, 'b>(&'a self, out: &'b mut Vec<&'a ContactManifold<P>>) {
        if self.manifold.len() != 0 {
            out.push(&self.manifold)
        }
    }
}
