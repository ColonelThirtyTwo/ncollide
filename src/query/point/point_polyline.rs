use crate::bounding_volume::AABB;
use crate::math::{Isometry, Point};
use na::{self, RealField};
use crate::partitioning::{BestFirstVisitStatus, BestFirstVisitor, BVH};
use crate::query::{
    visitors::CompositePointContainmentTest, PointProjection, PointQuery, PointQueryWithLocation,
};
use crate::shape::{FeatureId, Polyline, SegmentPointLocation};

impl<N: RealField> PointQuery<N> for Polyline<N> {
    #[inline]
    fn project_point(&self, m: &Isometry<N>, point: &Point<N>, solid: bool) -> PointProjection<N> {
        let (projection, _) = self.project_point_with_location(m, point, solid);
        projection
    }

    #[inline]
    fn project_point_with_feature(
        &self,
        _: &Isometry<N>,
        _: &Point<N>,
    ) -> (PointProjection<N>, FeatureId)
    {
        unimplemented!()
    }

    // FIXME: implement distance_to_point too?

    #[inline]
    fn contains_point(&self, m: &Isometry<N>, point: &Point<N>) -> bool {
        let ls_pt = m.inverse_transform_point(point);
        let mut visitor = CompositePointContainmentTest {
            shape: self,
            point: &ls_pt,
            found: false,
        };

        self.bvt().visit(&mut visitor);

        visitor.found
    }
}

impl<N: RealField> PointQueryWithLocation<N> for Polyline<N> {
    type Location = (usize, SegmentPointLocation<N>);

    #[inline]
    fn project_point_with_location(
        &self,
        m: &Isometry<N>,
        point: &Point<N>,
        _: bool,
    ) -> (PointProjection<N>, Self::Location)
    {
        let ls_pt = m.inverse_transform_point(point);
        let mut visitor = PolylinePointProjVisitor {
            polyline: self,
            point: &ls_pt,
        };

        let (mut proj, extra_info) = self.bvt().best_first_search(&mut visitor).unwrap().1;
        proj.point = m * proj.point;

        (proj, extra_info)
    }
}

/*
 * Visitors
 */
struct PolylinePointProjVisitor<'a, N: 'a + RealField> {
    polyline: &'a Polyline<N>,
    point: &'a Point<N>,
}

impl<'a, N: RealField> BestFirstVisitor<N, usize, AABB<N>> for PolylinePointProjVisitor<'a, N> {
    type Result = (PointProjection<N>, (usize, SegmentPointLocation<N>));

    #[inline]
    fn visit(&mut self, best: N, aabb: &AABB<N>, data: Option<&usize>) -> BestFirstVisitStatus<N, Self::Result> {
        let dist = aabb.distance_to_point(
            &Isometry::identity(),
            self.point,
            true,
        );

        let mut res = BestFirstVisitStatus::Continue { cost: dist, result: None };

        if let Some(b) = data {
            if dist < best {
                let (proj, extra_info) = self.polyline.segment_at(*b).project_point_with_location(
                    &Isometry::identity(),
                    self.point,
                    true,
                );

                let extra_info = (*b, extra_info);
                res = BestFirstVisitStatus::Continue {
                    cost: na::distance(self.point, &proj.point),
                    result: Some((proj, extra_info)),
                };
            }
        }

        res
    }
}
