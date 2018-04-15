use na::{self, Real};
use math::{Point, Isometry};
use query::{PointQuery, PointQueryWithLocation};
use query::algorithms::{CSOPoint, gjk, simplex::Simplex};
use shape::{Segment, SegmentPointLocation, Tetrahedron, TetrahedronPointLocation, Triangle,
            TrianglePointLocation};

/// A simplex of dimension up to 3 that uses Voronoï regions for computing point projections.
pub struct VoronoiSimplex<N: Real> {
    prev_vertices: [usize; 4],
    prev_proj: [N; 3],
    prev_dim: usize,

    vertices: [CSOPoint<N>; 4],
    proj: [N; 3],
    dim: usize,
}

impl<N: Real> VoronoiSimplex<N> {
    /// Creates a new empty simplex.
    pub fn new() -> VoronoiSimplex<N> {
        VoronoiSimplex {
            prev_vertices: [0, 1, 2, 3],
            prev_proj: [N::zero(); 3],
            prev_dim: 0,
            vertices: [CSOPoint::origin(); 4],
            proj: [N::zero(); 3],
            dim: 0,
        }
    }

    fn swap(&mut self, i1: usize, i2: usize) {
        self.vertices.swap(i1, i2);
        self.prev_vertices.swap(i1, i2);
        self.prev_proj.swap(i1, i2);
    }
}

/// Trait of a simplex usable by the GJK algorithm.
impl<N: Real> Simplex<N> for VoronoiSimplex<N> {
    fn reset(&mut self, pt: CSOPoint<N>) {
        self.dim = 0;
        self.prev_dim = 0;
        self.vertices[0] = pt;
    }

    fn add_point(&mut self, pt: CSOPoint<N>) -> bool {
        self.prev_dim = self.dim;
        self.prev_proj = self.proj;
        self.prev_vertices = [0, 1, 2, 3];

        match self.dim {
            0 => {
                if na::norm_squared(&(self.vertices[0] - pt)) < gjk::eps_tol() {
                    return false;
                }
            }
            1 => {
                let ab = self.vertices[1] - self.vertices[0];
                let ac = pt - self.vertices[0];

                if na::norm_squared(&ab.cross(&ac)) < gjk::eps_tol() {
                    return false;
                }
            }
            2 => {
                let ab = self.vertices[1] - self.vertices[0];
                let ac = self.vertices[2] - self.vertices[0];
                let ap = pt - self.vertices[0];
                let n = na::normalize(&ab.cross(&ac));

                if na::dot(&n, &ap).abs() < gjk::eps_tol() {
                    return false;
                }
            }
            _ => unreachable!(),
        }

        self.dim += 1;
        self.vertices[self.dim] = pt;
        return true;
    }

    fn proj_coord(&self, i: usize) -> N {
        assert!(i <= self.dim, "Index out of bounds.");
        self.proj[i]
    }

    fn point(&self, i: usize) -> &CSOPoint<N> {
        assert!(i <= self.dim, "Index out of bounds.");
        &self.vertices[i]
    }

    fn prev_proj_coord(&self, i: usize) -> N {
        assert!(i <= self.dim, "Index out of bounds.");
        self.prev_proj[i]
    }
    
    fn prev_point(&self, i: usize) -> &CSOPoint<N> {
        assert!(i <= self.prev_dim, "Index out of bounds.");
        &self.vertices[self.prev_vertices[i]]
    }

    fn project_origin_and_reduce(&mut self) -> Point<N> {
        if self.dim == 0 {
            self.proj[0] = N::one();
            self.vertices[0].point
        } else if self.dim == 1 {
            // FIXME: NLL
            let (proj, location) = {
                let seg = Segment::new(self.vertices[0].point, self.vertices[1].point);
                seg.project_point_with_location(&Isometry::identity(), &Point::origin(), true)
            };

            match location {
                SegmentPointLocation::OnVertex(0) => {
                    self.proj[0] = N::one();
                    self.dim = 0;
                }
                SegmentPointLocation::OnVertex(1) => {
                    self.swap(0, 1);
                    self.proj[0] = N::one();
                    self.dim = 0;
                }
                _ => {}
            }

            proj.point
        } else if self.dim == 2 {
            // FIXME: NLL
            let (proj, location) = {
                let tri = Triangle::new(self.vertices[0].point, self.vertices[1].point, self.vertices[2].point);
                tri.project_point_with_location(&Isometry::identity(), &Point::origin(), true)
            };

            match location {
                TrianglePointLocation::OnVertex(i) => {
                    self.swap(0, i);
                    self.proj[0] = N::one();
                    self.dim = 0;
                }
                TrianglePointLocation::OnEdge(0, coords) => {
                    self.proj[0] = coords[0];
                    self.proj[1] = coords[1];
                    self.dim = 1;
                }
                TrianglePointLocation::OnEdge(1, coords) => {
                    self.swap(0, 2);
                    self.proj[0] = coords[1];
                    self.proj[1] = coords[0];
                    self.dim = 1;
                }
                TrianglePointLocation::OnEdge(2, coords) => {
                    self.swap(1, 2);
                    self.proj[0] = coords[0];
                    self.proj[1] = coords[1];
                    self.dim = 1;
                }
                TrianglePointLocation::OnFace(coords) => {
                    self.proj = coords;
                }
                _ => {}
            }

            proj.point
        } else {
            assert!(self.dim == 3);
            // FIXME: NLL
            let (proj, location) = {
                let tetr = Tetrahedron::new(self.vertices[0].point, self.vertices[1].point, self.vertices[2].point, self.vertices[3].point);
                tetr.project_point_with_location(&Isometry::identity(), &Point::origin(), true)
            };

            match location {
                TetrahedronPointLocation::OnVertex(i) => {
                    self.swap(0, i);
                    self.proj[0] = N::one();
                    self.dim = 0;
                }
                TetrahedronPointLocation::OnEdge(i, coords) => {
                    match i {
                        0 => {
                            // ab
                        }
                        1 => {
                            // ac
                            self.swap(1, 2)
                        }
                        2 => {
                            // ad
                            self.swap(1, 3)
                        }
                        3 => {
                            // bc
                            self.swap(0, 2)
                        }
                        4 => {
                            // bd
                            self.swap(0, 3)
                        }
                        5 => {
                            // cd
                            self.swap(0, 2);
                            self.swap(1, 3);
                        }
                        _ => unreachable!(),
                    }

                     match i {
                        0 | 1 | 2 | 5 => {
                            self.proj[0] = coords[0];
                            self.proj[1] = coords[1];
                        }
                        3 | 4 => {
                            self.proj[0] = coords[1];
                            self.proj[1] = coords[0];
                        }
                        _ => unreachable!(),
                    }
                    self.dim = 1;
                }
                TetrahedronPointLocation::OnFace(i, coords) => {
                    match i {
                        0 => {
                            // abc
                            self.proj = coords;
                        }
                        1 => {
                            // abd
                            self.vertices[2] = self.vertices[3];
                            self.proj = coords;
                        }
                        2 => {
                            // acd
                            self.vertices[1] = self.vertices[3];
                            self.proj[0] = coords[0];
                            self.proj[1] = coords[2];
                            self.proj[2] = coords[1];
                        }
                        3 => {
                            // bcd
                            self.vertices[0] = self.vertices[3];
                            self.proj[0] = coords[2];
                            self.proj[1] = coords[0];
                            self.proj[2] = coords[1];
                        }
                        _ => unreachable!(),
                    }
                    self.dim = 2;
                }
                _ => {}
            }

            proj.point
        }
    }

    fn project_origin(&mut self) -> Point<N> {
        if self.dim == 0 {
            self.vertices[0].point
        } else if self.dim == 1 {
            let seg = Segment::new(self.vertices[0].point, self.vertices[1].point);
            seg.project_point(&Isometry::identity(), &Point::origin(), true).point
        } else if self.dim == 2 {
            let tri = Triangle::new(self.vertices[0].point, self.vertices[1].point, self.vertices[2].point);
            tri.project_point(&Isometry::identity(), &Point::origin(), true).point
        } else {
            let tetr = Tetrahedron::new(self.vertices[0].point, self.vertices[1].point, self.vertices[2].point, self.vertices[3].point);
            tetr.project_point(&Isometry::identity(), &Point::origin(), true).point
        }
    }

    fn contains_point(&self, pt: &Point<N>) -> bool {
        for i in 0..self.dim + 1 {
            if self.vertices[i].point == *pt {
                return true;
            }
        }

        false
    }

    fn dimension(&self) -> usize {
        self.dim
    }
    
    fn prev_dimension(&self) -> usize {
        self.prev_dim
    }

    fn max_sq_len(&self) -> N {
        let mut max_sq_len = na::zero();

        for i in 0..self.dim + 1 {
            let norm = na::norm_squared(&self.vertices[i].point.coords);

            if norm > max_sq_len {
                max_sq_len = norm
            }
        }

        max_sq_len
    }

    fn modify_pnts(&mut self, f: &Fn(&mut CSOPoint<N>)) {
        for i in 0..self.dim + 1 {
            f(&mut self.vertices[i])
        }
    }
}