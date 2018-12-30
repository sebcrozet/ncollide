use crate::bounding_volume::{self, BoundingVolume};
use crate::math::Isometry;
use na::{self, Real};
use crate::pipeline::narrow_phase::{ContactAlgorithm, ContactDispatcher, ContactManifoldGenerator, ConvexPolyhedronConvexPolyhedronManifoldGenerator};
use crate::query::{visitors::BoundingVolumeInterferencesCollector, ContactManifold, ContactPrediction, ContactPreprocessor, ContactTrackingMode};
use crate::shape::{Capsule, FeatureId, Shape};
use std::collections::{hash_map::Entry, HashMap};
use crate::utils::DeterministicState;
use crate::utils::IdAllocator;

/// Collision detector between a concave shape and another shape.
pub struct CapsuleCapsuleManifoldGenerator<N: Real> {
    // FIXME: use a dedicated segment-segment algorithm instead.
    sub_detector: ConvexPolyhedronConvexPolyhedronManifoldGenerator<N>
}

impl<N: Real> CapsuleCapsuleManifoldGenerator<N> {
    /// Creates a new collision detector between a concave shape and another shape.
    pub fn new() -> CapsuleCapsuleManifoldGenerator<N> {
        CapsuleCapsuleManifoldGenerator {
            sub_detector: ConvexPolyhedronConvexPolyhedronManifoldGenerator::new(),
        }
    }

    fn do_update(
        &mut self,
        dispatcher: &ContactDispatcher<N>,
        m1: &Isometry<N>,
        g1: &Capsule<N>,
        _proc1: Option<&ContactPreprocessor<N>>,
        m2: &Isometry<N>,
        g2: &Capsule<N>,
        _proc2: Option<&ContactPreprocessor<N>>,
        prediction: &ContactPrediction<N>,
        id_alloc: &mut IdAllocator,
        manifold: &mut ContactManifold<N>
    ) -> bool
    {
        let segment1 = g1.segment();
        let segment2 = g2.segment();

        let mut prediction = prediction.clone();
        let new_linear_prediction = prediction.linear() + g1.radius() + g2.radius();
        prediction.set_linear(new_linear_prediction);

        // Update all collisions
        self.sub_detector.generate_contacts(
            dispatcher,
            m1,
            &segment1,
            Some(&g1.contact_preprocessor()),
            m2,
            &segment2,
            Some(&g2.contact_preprocessor()),
            &prediction,
            id_alloc,
            manifold
        )
    }
}

impl<N: Real> ContactManifoldGenerator<N> for CapsuleCapsuleManifoldGenerator<N> {
    fn generate_contacts(
        &mut self,
        d: &ContactDispatcher<N>,
        ma: &Isometry<N>,
        a: &Shape<N>,
        proc1: Option<&ContactPreprocessor<N>>,
        mb: &Isometry<N>,
        b: &Shape<N>,
        proc2: Option<&ContactPreprocessor<N>>,
        prediction: &ContactPrediction<N>,
        id_alloc: &mut IdAllocator,
        manifold: &mut ContactManifold<N>,
    ) -> bool
    {
        if let (Some(cs1), Some(cs2)) = (a.as_shape::<Capsule<N>>(), b.as_shape::<Capsule<N>>()) {
            self.do_update(d, ma, cs1, proc1, mb, cs2, proc2, prediction, id_alloc, manifold)
        } else {
            false
        }
    }
}