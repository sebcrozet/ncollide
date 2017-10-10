use na;
use math::Point;

/// Description of the projection of a point on a shape.
pub struct PointProjection<P: Point> {
    /// Whether or not the point to project was inside of the shape.
    pub is_inside: bool,
    /// The projection result.
    pub point: P,
}

impl<P: Point> PointProjection<P> {
    /// Initializes a new `PointProjection`.
    #[inline]
    pub fn new(is_inside: bool, point: P) -> PointProjection<P> {
        PointProjection {
            is_inside: is_inside,
            point:     point
        }
    }
}

/// Trait of objects that can be tested for point inclusion and projection.
pub trait PointQuery<P: Point, M> {
    /// Projects a point on `self` transformed by `m`.
    fn project_point(&self, m: &M, pt: &P, solid: bool) -> PointProjection<P>;

    /// Computes the minimal distance between a point and `self` transformed by `m`.
    #[inline]
    fn distance_to_point(&self, m: &M, pt: &P, solid: bool) -> P::Real {
        let proj = self.project_point(m, pt, solid);
        let dist = na::distance(pt, &proj.point);

        if solid || !proj.is_inside {
            dist
        }
        else {
            -dist
        }
    }

    /// Tests if the given point is inside of `self` transformed by `m`.
    #[inline]
    fn contains_point(&self, m: &M, pt: &P) -> bool {
        self.project_point(m, pt, false).is_inside
    }
}

/// Returns shape-specific info in addition to generic projection information
///
/// One requirement for the `PointQuery` trait is to be usable as a trait
/// object. Unfortunately this precludes us from adding an associated type to it
/// that might allow us to return shape-specific information in addition to the
/// general information provided in `PointProjection`. This is where
/// `RichPointQuery` comes in. It forgoes the ability to be used as a trait
/// object in exchange for being able to provide shape-specific projection
/// information.
///
/// Any shapes that implement `PointQuery` but are able to provide extra
/// information, can implement `RichPointQuery` in addition and have their
/// `PointQuery::project_point` implementation just call out to
/// `RichPointQuery::project_point_with_extra_info`.
pub trait RichPointQuery<P: Point, M> {
    /// Additional shape-specific projection information
    ///
    /// In addition to the generic projection information returned in
    /// `PointProjection`, implementations might provide shape-specific
    /// projection info. The type of this shape-specific information is defined
    /// by this associated type.
    type ExtraInfo;

    /// Projects a point on `self` transformed by `m`.
    fn project_point_with_extra_info(&self, m: &M, pt: &P, solid: bool)
        -> (PointProjection<P>, Self::ExtraInfo);
}
