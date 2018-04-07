use std::marker::PhantomData;
use alga::linear::Translation;
use na::{self, Unit};

use math::{Isometry, Point};
use utils::IdAllocator;
use geometry::shape::{ConvexPolyface, FeatureId, Plane, Shape};
use geometry::bounding_volume::PolyhedralCone;
use geometry::query::{Contact, ContactKinematic, ContactManifold, ContactPrediction};
use narrow_phase::{ContactDispatcher, ContactManifoldGenerator};

/// Collision detector between g1 plane and g1 shape implementing the `SupportMap` trait.
#[derive(Clone)]
pub struct PlaneConvexPolyhedronManifoldGenerator<P: Point, M> {
    flip: bool,
    feature: ConvexPolyface<P>,
    manifold: ContactManifold<P>,
    mat_type: PhantomData<M>, // FIXME: can we avoid this?
}

impl<P: Point, M: Isometry<P>> PlaneConvexPolyhedronManifoldGenerator<P, M> {
    /// Creates g1 new persistent collision detector between g1 plane and g1 shape with g1 support
    /// mapping function.
    #[inline]
    pub fn new(flip: bool) -> PlaneConvexPolyhedronManifoldGenerator<P, M> {
        PlaneConvexPolyhedronManifoldGenerator {
            flip,
            feature: ConvexPolyface::new(),
            manifold: ContactManifold::new(),
            mat_type: PhantomData,
        }
    }

    #[inline]
    fn do_update(
        &mut self,
        m1: &M,
        g1: &Shape<P, M>,
        m2: &M,
        g2: &Shape<P, M>,
        prediction: &ContactPrediction<P::Real>,
        id_alloc: &mut IdAllocator,
        flip: bool,
    ) -> bool {
        if let (Some(plane), Some(cp)) =
            (g1.as_shape::<Plane<P::Vector>>(), g2.as_convex_polyhedron())
        {
            self.manifold.save_cache_and_clear(id_alloc);
            let plane_normal = m1.transform_unit_vector(plane.normal());
            let plane_center = P::from_coordinates(m1.translation().to_vector());

            cp.support_face_toward(m2, &-plane_normal, &mut self.feature);

            for (i, world2) in self.feature.vertices.iter().enumerate() {
                let dpt = *world2 - plane_center;
                let dist = na::dot(&dpt, plane_normal.as_ref());

                if dist <= prediction.linear {
                    let world1 = *world2 + (-*plane_normal * dist);
                    let local1 = m1.inverse_transform_point(&world1);
                    let local2 = m2.inverse_transform_point(&world2);
                    let f1 = FeatureId::Face(0);
                    let f2 = self.feature.vertices_id[i];
                    let n2 = cp.normal_cone(f2);
                    let mut kinematic = ContactKinematic::new();
                    let contact;

                    if !flip {
                        contact = Contact::new(world1, *world2, plane_normal, -dist);
                        kinematic.set_plane1(f1, local1, *plane.normal());
                        kinematic.set_point2(f2, local2, n2);
                    } else {
                        contact = Contact::new(*world2, world1, -plane_normal, -dist);
                        kinematic.set_point1(f2, local2, n2);
                        kinematic.set_plane2(f1, local1, *plane.normal());
                    }
                    let _ = self.manifold.push(contact, kinematic, id_alloc);
                }
            }

            true
        } else {
            false
        }
    }
}

impl<P: Point, M: Isometry<P>> ContactManifoldGenerator<P, M>
    for PlaneConvexPolyhedronManifoldGenerator<P, M>
{
    #[inline]
    fn update(
        &mut self,
        _: &ContactDispatcher<P, M>,
        id1: usize,
        m1: &M,
        g1: &Shape<P, M>,
        id2: usize,
        m2: &M,
        g2: &Shape<P, M>,
        prediction: &ContactPrediction<P::Real>,
        id_alloc: &mut IdAllocator,
    ) -> bool {
        self.manifold.set_subshape_id1(id1);
        self.manifold.set_subshape_id2(id2);

        if !self.flip {
            self.do_update(m1, g1, m2, g2, prediction, id_alloc, false)
        } else {
            self.do_update(m2, g2, m1, g1, prediction, id_alloc, true)
        }
    }

    #[inline]
    fn num_contacts(&self) -> usize {
        self.manifold.len()
    }

    #[inline]
    fn contacts<'a: 'b, 'b>(&'a self, out: &'b mut Vec<&'a ContactManifold<P>>) {
        out.push(&self.manifold)
    }
}