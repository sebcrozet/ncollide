//!
//! Support mapping based Ball geometry.
//!

use nalgebra::traits::translation::Translation;
use nalgebra::traits::vector::{AlgebraicVec, AlgebraicVecExt};
use nalgebra::traits::scalar_op::{ScalarSub, ScalarAdd};
use geom::implicit::Implicit;
use bounding_volume::aabb::{HasAABB, AABB};

/**
 * Implicit description of a ball geometry.
 * 
 *  - `N`: numeric type used for the ball radius.
 */
#[deriving(Eq, ToStr, Clone)]
pub struct Ball<N> {
    priv radius: N
}

impl<N> Ball<N> {
    /**
     * Creates a new ball from its radius and center.
     */
    #[inline]
    pub fn new(radius: N) -> Ball<N> {
        Ball { radius: radius }
    }
}

impl<N: Clone> Ball<N> {
    /**
     * The ball radius.
     */
    #[inline]
    pub fn radius(&self) -> N {
        self.radius.clone()
    }
}

impl<N: Algebraic, V: AlgebraicVec<N>, M: Translation<V>> Implicit<V, M> for Ball<N> {
    #[inline]
    fn support_point(&self, m: &M, dir: &V) -> V {
        m.translation() + dir.normalized() * self.radius
    }
}

impl<N,
     V: AlgebraicVecExt<N> + Ord,
     M: Translation<V>>
HasAABB<N, V, M> for Ball<N> {
    #[inline]
    fn aabb(&self, m: &M) -> AABB<N, V> {
        AABB::new(m.translation().scalar_sub(&self.radius),
                  m.translation().scalar_add(&self.radius))
    }
}
