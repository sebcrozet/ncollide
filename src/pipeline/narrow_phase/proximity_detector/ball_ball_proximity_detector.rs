use crate::math::{Isometry, Point};
use na::RealField;
use crate::pipeline::narrow_phase::{ProximityDetector, ProximityDispatcher};
use crate::query::{self, Proximity};
use crate::shape::{Ball, Shape};

/// Proximity detector between two balls.
pub struct BallBallProximityDetector {
    proximity: Proximity,
}

impl Clone for BallBallProximityDetector {
    fn clone(&self) -> BallBallProximityDetector {
        BallBallProximityDetector {
            proximity: self.proximity,
        }
    }
}

impl BallBallProximityDetector {
    /// Creates a new persistent collision detector between two balls.
    #[inline]
    pub fn new() -> BallBallProximityDetector {
        BallBallProximityDetector {
            proximity: Proximity::Disjoint,
        }
    }
}

impl<N: RealField> ProximityDetector<N> for BallBallProximityDetector {
    fn update(
        &mut self,
        _: &ProximityDispatcher<N>,
        ma: &Isometry<N>,
        a: &Shape<N>,
        mb: &Isometry<N>,
        b: &Shape<N>,
        margin: N,
    ) -> bool
    {
        if let (Some(a), Some(b)) = (a.as_shape::<Ball<N>>(), b.as_shape::<Ball<N>>()) {
            self.proximity = query::proximity_ball_ball(
                &Point::from(ma.translation.vector),
                a,
                &Point::from(mb.translation.vector),
                b,
                margin,
            );

            true
        } else {
            false
        }
    }

    #[inline]
    fn proximity(&self) -> Proximity {
        self.proximity
    }
}