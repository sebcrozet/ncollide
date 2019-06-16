use na::RealField;

use crate::utils::IsometryOps;
use crate::math::{Isometry, Vector, Rotation};


/// A continuous interpolation of isometries.
pub trait RigidMotion<N: RealField> {
    /// Get a position at the time `t`.
    fn position_at_time(&self, t: N) -> Isometry<N>;
}


/// Interpolation between two isometries using LERP for the translation part and SLERP for the rotation.
pub struct InterpolatedRigidMotion<'a, N: RealField> {
    /// The transformation at `t = 0.0`.
    pub start: &'a Isometry<N>,
    /// The transformation at `t = 1.0`.
    pub end: &'a Isometry<N>,
}

impl<'a, N: RealField> InterpolatedRigidMotion<'a, N> {
    /// Initialize a lerp-slerp interpolation with the given start and end transformations.
    ///
    /// The `start` is the transformation at the time `t = 0.0` and `end` is the transformation at
    /// the time `t = 1.0`.
    pub fn new(start: &'a Isometry<N>, end: &'a Isometry<N>) -> Self {
        InterpolatedRigidMotion {
            start, end
        }
    }
}

impl<'a, N: RealField> RigidMotion<N> for InterpolatedRigidMotion<'a, N> {
    fn position_at_time(&self, t: N) -> Isometry<N> {
        self.start.lerp_slerp(self.end, t)
    }
}

/// A linear motion from a starting isometry traveling at constant translational velocity.
pub struct ConstantLinearVelocityRigidMotion<'a, N: RealField> {
    /// The starting isometry at `t = 0`.
    pub start: &'a Isometry<N>,
    /// The translational velocity of this motion.
    pub velocity: Vector<N>,
}

impl<'a, N: RealField> ConstantLinearVelocityRigidMotion<'a, N> {
    /// Initialize a linear motion frow a starting isometry and a translational velocity.
    pub fn new(start: &'a Isometry<N>, velocity: Vector<N>) -> Self {
        ConstantLinearVelocityRigidMotion {
            start, velocity
        }
    }
}

impl<'a, N: RealField> RigidMotion<N> for ConstantLinearVelocityRigidMotion<'a, N> {
    fn position_at_time(&self, t: N) -> Isometry<N> {
        Isometry::from_parts(
            (self.start.translation.vector + self.velocity * t).into(),
            self.start.rotation
        )
    }
}


/// A linear motion from a starting isometry traveling at constant translational velocity.
pub struct ConstantVelocityRigidMotion<'a, N: RealField> {
    /// The starting isometry at `t = 0`.
    pub start: &'a Isometry<N>,
    /// The translational velocity of this motion.
    pub linvel: Vector<N>,
    /// The angular velocity of this motion.
    #[cfg(feature = "dim2")]
    pub angvel: N,
    /// The angular velocity of this motion.
    #[cfg(feature = "dim3")]
    pub angvel: Vector<N>,

}

impl<'a, N: RealField> ConstantVelocityRigidMotion<'a, N> {
    /// Initialize a motion from a starting isometry and linear and angular velocities.
    #[cfg(feature = "dim2")]
    pub fn new(start: &'a Isometry<N>, linvel: Vector<N>, angvel: N) -> Self {
        ConstantVelocityRigidMotion {
            start, linvel, angvel
        }
    }

    /// Initialize a motion from a starting isometry and linear and angular velocities.
    #[cfg(feature = "dim3")]
    pub fn new(start: &'a Isometry<N>, linvel: Vector<N>, angvel: Vector<N>) -> Self {
        ConstantVelocityRigidMotion {
            start, linvel, angvel
        }
    }
}

impl<'a, N: RealField> RigidMotion<N> for ConstantVelocityRigidMotion<'a, N> {
    fn position_at_time(&self, t: N) -> Isometry<N> {
        Isometry::from_parts(
            (self.start.translation.vector + self.linvel * t).into(),
            Rotation::new(self.angvel * t) * self.start.rotation
        )
    }
}