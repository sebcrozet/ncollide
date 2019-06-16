//! Implementation details of the `contact` and `contacts` functions.

pub use self::contact::{Contact, ContactPrediction, TrackedContact, ContactId};
pub use self::contact_kinematic::{
    ContactKinematic, LocalShapeApproximation, NeighborhoodGeometry,
};
pub use self::contact_manifold::{ContactManifold, ContactTrackingMode};
pub use self::contact_preprocessor::ContactPreprocessor;


pub use self::contact_ball_ball::contact_ball_ball;
pub use self::contact_composite_shape_shape::{
    contact_composite_shape_shape, contact_shape_composite_shape,
};
pub use self::contact_plane_support_map::{contact_plane_support_map, contact_support_map_plane};
pub use self::contact_shape_shape::contact;
pub use self::contact_support_map_support_map::contact_support_map_support_map;
pub use self::contact_support_map_support_map::contact_support_map_support_map_with_params;

mod contact_ball_ball;
mod contact_composite_shape_shape;
mod contact;
mod contact_kinematic;
mod contact_manifold;
mod contact_plane_support_map;
mod contact_shape_shape;
mod contact_support_map_support_map;
mod contact_preprocessor;