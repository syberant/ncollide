use crate::bounding_volume::AABB;
use crate::math::{Isometry, Point, Vector};
use na::{self, RealField};
use crate::partitioning::{BestFirstBVVisitStatus, BestFirstDataVisitStatus, BestFirstVisitor};
use crate::query::distance_internal;
use crate::query::PointQuery;
use crate::shape::{CompositeShape, Shape};

/// Smallest distance between a composite shape and any other shape.
pub fn composite_shape_against_shape<N, G1: ?Sized>(
    m1: &Isometry<N>,
    g1: &G1,
    m2: &Isometry<N>,
    g2: &dyn Shape<N>,
) -> N
where
    N: RealField,
    G1: CompositeShape<N>,
{
    let ls_m2 = m1.inverse() * m2.clone();
    let ls_aabb2 = g2.aabb(&ls_m2);

    let mut visitor = CompositeShapeAgainstAnyDistanceVisitor {
        msum_shift: -ls_aabb2.center().coords,
        msum_margin: ls_aabb2.half_extents(),
        m1: m1,
        g1: g1,
        m2: m2,
        g2: g2,
    };

    g1.bvh()
        .best_first_search(&mut visitor)
        .expect("The composite shape must not be empty.")
}

/// Smallest distance between a shape and a composite shape.
pub fn shape_against_composite_shape<N, G2: ?Sized>(
    m1: &Isometry<N>,
    g1: &dyn Shape<N>,
    m2: &Isometry<N>,
    g2: &G2,
) -> N
where
    N: RealField,
    G2: CompositeShape<N>,
{
    composite_shape_against_shape(m2, g2, m1, g1)
}

struct CompositeShapeAgainstAnyDistanceVisitor<'a, N: 'a + RealField, G1: ?Sized + 'a> {
    msum_shift: Vector<N>,
    msum_margin: Vector<N>,

    m1: &'a Isometry<N>,
    g1: &'a G1,
    m2: &'a Isometry<N>,
    g2: &'a dyn Shape<N>,
}

impl<'a, N, G1: ?Sized> BestFirstVisitor<N, usize, AABB<N>>
    for CompositeShapeAgainstAnyDistanceVisitor<'a, N, G1>
where
    N: RealField,
    G1: CompositeShape<N>,
{
    type Result = N;

    fn visit_bv(&mut self, bv: &AABB<N>) -> BestFirstBVVisitStatus<N> {
        // Compute the minkowski sum of the two AABBs.
        let msum = AABB::new(
            *bv.mins() + self.msum_shift + (-self.msum_margin),
            *bv.maxs() + self.msum_shift + self.msum_margin,
        );

        // Compute the distance to the origin.
        BestFirstBVVisitStatus::ContinueWithCost(msum.distance_to_point(
            &Isometry::identity(),
            &Point::origin(),
            true,
        ))
    }

    fn visit_data(&mut self, b: &usize) -> BestFirstDataVisitStatus<N, N> {
        let mut res = BestFirstDataVisitStatus::Continue;

        self.g1
            .map_part_at(*b, self.m1, &mut |m1, g1| {
                let distance = distance_internal::distance(m1, g1, self.m2, self.g2);
                res = BestFirstDataVisitStatus::ContinueWithResult(distance, distance)
            });

        res
    }
}
