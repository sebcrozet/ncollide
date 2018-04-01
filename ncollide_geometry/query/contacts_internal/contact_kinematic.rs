use approx::ApproxEq;
use na::{self, Unit};
use shape::FeatureId;
use bounding_volume::PolyhedralCone;
use query::Contact;
use query::closest_points_internal;
use math::{Isometry, Point};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum KinematicVariant<P: Point> {
    PlanePoint,
    PointPlane,
    PointPoint,
    PointLine(Unit<P::Vector>),
    LinePoint(Unit<P::Vector>),
    LineLine(Unit<P::Vector>, Unit<P::Vector>),

    // NOTE: invalid cases
    PlanePlane,
    PlaneLine(Unit<P::Vector>),
    LinePlane(Unit<P::Vector>),
}

#[derive(Clone, Debug)]
pub struct ContactKinematic<P: Point> {
    local1: P,
    local2: P,

    normals1: PolyhedralCone<P>,
    normals2: PolyhedralCone<P>,

    margin1: P::Real,
    margin2: P::Real,

    feature1: FeatureId,
    feature2: FeatureId,

    variant: KinematicVariant<P>,
}

impl<P: Point> ContactKinematic<P> {
    pub fn new() -> Self {
        ContactKinematic {
            local1: P::origin(),
            local2: P::origin(),
            margin1: na::zero(),
            margin2: na::zero(),
            normals1: PolyhedralCone::new(),
            normals2: PolyhedralCone::new(),
            feature1: FeatureId::Unknown,
            feature2: FeatureId::Unknown,
            variant: KinematicVariant::PointPoint,
        }
    }

    pub fn transform1<M: Isometry<P>>(&mut self, m: &M) {
        self.local1 = m.transform_point(&self.local1);
        self.normals1.transform_by(m);

        match self.variant {
            KinematicVariant::LinePoint(ref mut dir)
            | KinematicVariant::LineLine(ref mut dir, _)
            | KinematicVariant::LinePlane(ref mut dir) => {
                *dir = m.transform_unit_vector(dir);
            }
            // We don't use the _ here pattern to we make sure
            // we don't forget to change this if we add other variants.
            KinematicVariant::PointLine(_)
            | KinematicVariant::PlaneLine(_)
            | KinematicVariant::PointPlane
            | KinematicVariant::PointPoint
            | KinematicVariant::PlanePoint
            | KinematicVariant::PlanePlane => {}
        }
    }

    pub fn transform2<M: Isometry<P>>(&mut self, m: &M) {
        self.local2 = m.transform_point(&self.local2);
        self.normals2.transform_by(m);

        match self.variant {
            KinematicVariant::PointLine(ref mut dir)
            | KinematicVariant::LineLine(_, ref mut dir)
            | KinematicVariant::PlaneLine(ref mut dir) => {
                *dir = m.transform_unit_vector(dir);
            }
            // We don't use the _ pattern here to we make sure
            // we don't forget to change this if we add other variants.
            KinematicVariant::PointPlane
            | KinematicVariant::PointPoint
            | KinematicVariant::LinePoint(..)
            | KinematicVariant::LinePlane(_)
            | KinematicVariant::PlanePoint
            | KinematicVariant::PlanePlane => {}
        }
    }

    pub fn dilation1(&self) -> P::Real {
        self.margin1
    }

    pub fn dilation2(&self) -> P::Real {
        self.margin2
    }

    /// The tracked point in local space of the first solid.
    ///
    /// This may not correspond to the contact point in the local
    /// space of the first since it does not takes the dilation
    /// into account.
    pub fn local1(&self) -> P {
        self.local1
    }

    /// The tracked point in local space of the second solid.
    ///
    /// This may not correspond to the contact point in the local
    /// space of the second solid since it does not takes the dilation
    /// into account.
    pub fn local2(&self) -> P {
        self.local2
    }

    pub fn feature1(&self) -> FeatureId {
        self.feature1
    }

    pub fn feature2(&self) -> FeatureId {
        self.feature2
    }

    pub fn set_dilation1(&mut self, margin: P::Real) {
        self.margin1 = margin;
    }

    pub fn set_dilation2(&mut self, margin: P::Real) {
        self.margin2 = margin;
    }

    pub fn set_plane1(&mut self, fid: FeatureId, pt: P, normal: Unit<P::Vector>) {
        self.feature1 = fid;
        self.local1 = pt;
        self.normals1.clear();
        self.normals1.add_generator(normal);

        self.variant = match self.variant {
            KinematicVariant::PointPlane => KinematicVariant::PlanePlane,
            KinematicVariant::PointPoint => KinematicVariant::PlanePoint,
            KinematicVariant::PointLine(dir2) => KinematicVariant::PlaneLine(dir2),
            KinematicVariant::LinePoint(..) => KinematicVariant::PlanePoint,
            KinematicVariant::LineLine(_, dir2) => KinematicVariant::PlaneLine(dir2),
            KinematicVariant::LinePlane(_) => KinematicVariant::PlanePlane,
            // Other cases don't change anthying.
            KinematicVariant::PlanePoint
            | KinematicVariant::PlaneLine(_)
            | KinematicVariant::PlanePlane => self.variant,
        }
    }

    pub fn set_plane2(&mut self, fid: FeatureId, pt: P, normal: Unit<P::Vector>) {
        self.feature2 = fid;
        self.local2 = pt;
        self.normals2.clear();
        self.normals2.add_generator(normal);

        self.variant = match self.variant {
            KinematicVariant::PlanePoint => KinematicVariant::PlanePlane,
            KinematicVariant::PointPoint => KinematicVariant::PointPlane,
            KinematicVariant::PointLine(_) => KinematicVariant::PointPlane,
            KinematicVariant::LinePoint(dir1) => KinematicVariant::LinePlane(dir1),
            KinematicVariant::LineLine(dir1, _) => KinematicVariant::LinePlane(dir1),
            KinematicVariant::PlaneLine(_) => KinematicVariant::PlanePlane,
            // Other cases don't change anthying.
            KinematicVariant::PointPlane
            | KinematicVariant::PlanePlane
            | KinematicVariant::LinePlane(_) => self.variant,
        }
    }

    pub fn set_line1(
        &mut self,
        fid: FeatureId,
        pt: P,
        dir: Unit<P::Vector>,
        normals: PolyhedralCone<P>,
    ) {
        self.feature1 = fid;
        self.local1 = pt;
        self.normals1 = normals;

        self.variant = match self.variant {
            KinematicVariant::PlanePoint => KinematicVariant::LinePoint(dir),
            KinematicVariant::PointPlane => KinematicVariant::LinePlane(dir),
            KinematicVariant::PointPoint => KinematicVariant::LinePoint(dir),
            KinematicVariant::PointLine(dir2) => KinematicVariant::LineLine(dir, dir2),
            KinematicVariant::LinePoint(_) => KinematicVariant::LinePoint(dir),
            KinematicVariant::LineLine(_, dir2) => KinematicVariant::LineLine(dir, dir2),
            KinematicVariant::PlanePlane => KinematicVariant::LinePlane(dir),
            KinematicVariant::PlaneLine(dir2) => KinematicVariant::LineLine(dir, dir2),
            KinematicVariant::LinePlane(_) => KinematicVariant::LinePlane(dir),
        };
    }

    pub fn set_line2(
        &mut self,
        fid: FeatureId,
        pt: P,
        dir: Unit<P::Vector>,
        normals: PolyhedralCone<P>,
    ) {
        self.feature2 = fid;
        self.local2 = pt;
        self.normals2 = normals;

        self.variant = match self.variant {
            KinematicVariant::PlanePoint => KinematicVariant::PlaneLine(dir),
            KinematicVariant::PointPlane => KinematicVariant::PointLine(dir),
            KinematicVariant::PointPoint => KinematicVariant::PointLine(dir),
            KinematicVariant::PointLine(_) => KinematicVariant::PointLine(dir),
            KinematicVariant::LinePoint(dir1) => KinematicVariant::LineLine(dir1, dir),
            KinematicVariant::LineLine(dir1, _) => KinematicVariant::LineLine(dir1, dir),
            KinematicVariant::PlanePlane => KinematicVariant::PlaneLine(dir),
            KinematicVariant::PlaneLine(_) => KinematicVariant::PlaneLine(dir),
            KinematicVariant::LinePlane(dir1) => KinematicVariant::LineLine(dir1, dir),
        };
    }

    pub fn set_point1(&mut self, fid: FeatureId, pt: P, normals: PolyhedralCone<P>) {
        self.feature1 = fid;
        self.local1 = pt;
        self.normals1 = normals;

        self.variant = match self.variant {
            KinematicVariant::PlanePoint => KinematicVariant::PointPoint,
            KinematicVariant::LinePoint(_) => KinematicVariant::PointPoint,
            KinematicVariant::LineLine(_, dir2) => KinematicVariant::PointLine(dir2),
            KinematicVariant::PlanePlane => KinematicVariant::PointPlane,
            KinematicVariant::PlaneLine(dir2) => KinematicVariant::PointLine(dir2),
            KinematicVariant::LinePlane(_) => KinematicVariant::PointPlane,
            // Other cases don't change anthying.
            KinematicVariant::PointPlane
            | KinematicVariant::PointLine(_)
            | KinematicVariant::PointPoint => self.variant,
        };
    }

    pub fn set_point2(&mut self, fid: FeatureId, pt: P, normals: PolyhedralCone<P>) {
        self.feature2 = fid;
        self.local2 = pt;
        self.normals2 = normals;

        self.variant = match self.variant {
            KinematicVariant::PointPlane => KinematicVariant::PointPoint,
            KinematicVariant::PointLine(_) => KinematicVariant::PointPoint,
            KinematicVariant::LineLine(dir1, _) => KinematicVariant::LinePoint(dir1),
            KinematicVariant::PlanePlane => KinematicVariant::PlanePoint,
            KinematicVariant::PlaneLine(_) => KinematicVariant::PlanePoint,
            KinematicVariant::LinePlane(dir1) => KinematicVariant::LinePoint(dir1),
            // Other cases don't change anthying.
            KinematicVariant::PlanePoint
            | KinematicVariant::PointPoint
            | KinematicVariant::LinePoint(_) => self.variant,
        };
    }

    /// Computes the updated contact points with the new positions of the solids.
    ///
    /// The vector `default_normal1` is the normal of the resulting contactc
    /// in the rare case where the contact normal cannot be determined by the update.
    /// Typically, this should be set to the latest contact normal known.
    pub fn contact<M: Isometry<P>>(
        &self,
        m1: &M,
        m2: &M,
        default_normal1: &Unit<P::Vector>,
    ) -> Option<Contact<P>> {
        let mut world1 = m1.transform_point(&self.local1);
        let mut world2 = m2.transform_point(&self.local2);
        let normal;
        let mut depth;

        match self.variant {
            KinematicVariant::PlanePoint => {
                normal = m1.transform_unit_vector(&self.normals1.generators()[0]);
                depth = -na::dot(normal.as_ref(), &(world2 - world1));
                world1 = world2 + *normal * depth;
            }
            KinematicVariant::PointPlane => {
                let world_normal2 = m2.transform_unit_vector(&self.normals2.generators()[0]);
                depth = -na::dot(&*world_normal2, &(world1 - world2));
                world2 = world1 + *world_normal2 * depth;
                normal = -world_normal2;
            }
            KinematicVariant::PointPoint => {
                if let Some((n, d)) =
                    Unit::try_new_and_get(world2 - world1, P::Real::default_epsilon())
                {
                    let local_n1 = m1.inverse_transform_unit_vector(&n);
                    let local_n2 = m2.inverse_transform_unit_vector(&-n);
                    if self.normals1.polar_contains_dir(&local_n1)
                        || self.normals2.polar_contains_dir(&local_n2)
                    {
                        depth = d;
                        normal = -n;
                    } else {
                        depth = -d;
                        normal = n;
                    }
                } else {
                    depth = na::zero();
                    normal = m1.transform_unit_vector(default_normal1);
                }
            }
            KinematicVariant::LinePoint(ref dir1) => {
                let world_dir1 = m1.transform_unit_vector(dir1);
                let mut shift = world2 - world1;
                let proj = na::dot(world_dir1.as_ref(), &shift);
                shift -= dir1.unwrap() * proj;

                if let Some((n, d)) = Unit::try_new_and_get(shift, na::zero()) {
                    let local_n1 = m1.inverse_transform_unit_vector(&n);
                    let local_n2 = m2.inverse_transform_unit_vector(&-n);
                    world1 = world2 + (-shift);

                    if self.normals1.polar_contains_dir(&local_n1)
                        || self.normals2.polar_contains_dir(&local_n2)
                    {
                        depth = d;
                        normal = -n;
                    } else {
                        depth = -d;
                        normal = n;
                    }
                } else {
                    depth = na::zero();
                    normal = m1.transform_unit_vector(default_normal1);
                }
            }
            KinematicVariant::PointLine(ref dir2) => {
                let world_dir2 = m2.transform_unit_vector(dir2);
                let mut shift = world1 - world2;
                let proj = na::dot(world_dir2.as_ref(), &shift);
                shift -= dir2.unwrap() * proj;
                // NOTE: we set:
                // shift = world2 - world1
                let shift = -shift;

                if let Some((n, d)) = Unit::try_new_and_get(shift, na::zero()) {
                    let local_n1 = m1.inverse_transform_unit_vector(&n);
                    let local_n2 = m2.inverse_transform_unit_vector(&-n);
                    world2 = world1 + shift;

                    if self.normals1.polar_contains_dir(&local_n1)
                        || self.normals2.polar_contains_dir(&local_n2)
                    {
                        depth = d;
                        normal = -n;
                    } else {
                        depth = -d;
                        normal = n;
                    }
                } else {
                    depth = na::zero();
                    normal = m1.transform_unit_vector(default_normal1);
                }
            }
            KinematicVariant::LineLine(ref dir1, ref dir2) => {
                let world_dir1 = m1.transform_unit_vector(dir1);
                let world_dir2 = m2.transform_unit_vector(dir2);
                let (pt1, pt2) = closest_points_internal::line_against_line(
                    &world1,
                    &world_dir1,
                    &world2,
                    &world_dir2,
                );

                world1 = pt1;
                world2 = pt2;

                if let Some((n, d)) = Unit::try_new_and_get(world2 - world1, na::zero()) {
                    let local_n1 = m1.inverse_transform_unit_vector(&n);
                    let local_n2 = m2.inverse_transform_unit_vector(&-n);

                    if self.normals1.polar_contains_dir(&local_n1)
                        || self.normals2.polar_contains_dir(&local_n2)
                    {
                        depth = d;
                        normal = -n;
                    } else {
                        depth = -d;
                        normal = n;
                    }
                } else {
                    depth = na::zero();
                    normal = m1.transform_unit_vector(default_normal1);
                }
            }
            _ => {
                return None;
            }
        }

        world1 += normal.unwrap() * self.margin1;
        world2 += normal.unwrap() * (-self.margin2);
        depth += self.margin1 + self.margin2;

        Some(Contact::new(world1, world2, normal, depth))
    }
}
