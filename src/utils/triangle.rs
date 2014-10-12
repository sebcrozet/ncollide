use std::num::Zero;
use na::{Cast, FloatVec, FloatPnt, FloatPntExt, Cross, ApproxEq, Norm};
use na;
use bounding_volume;
use utils;

/// Computes the area of a triangle.
#[inline]
pub fn triangle_area<N: Float + Cast<f64>, P: FloatPnt<N, V>, V: FloatVec<N>>(pa: &P, pb: &P, pc: &P) -> N {
    // Kahan's formula.
    let mut a = na::dist(pa, pb);
    let mut b = na::dist(pb, pc);
    let mut c = na::dist(pc, pa);

    let (c, b, a) = utils::sort3(&mut a, &mut b, &mut c);
    let a = *a;
    let b = *b;
    let c = *c;

    let sqr = (a + (b + c)) * (c - (a - b)) * (c + (a - b)) * (a + (b - c));

    sqr.sqrt() * na::cast(0.25)
}

/// Computes the perimeter of a triangle.
#[inline]
pub fn triangle_perimeter<N: Float, P: FloatPnt<N, V>, V: FloatVec<N>>(pa: &P, pb: &P, pc: &P) -> N {
    na::dist(pa, pb) + na::dist(pb, pc) + na::dist(pc, pa)
}

/// Computes the circumcircle of a triangle.
pub fn circumcircle<N: Float + Cast<f64>,
                    P: FloatPntExt<N, V> + Clone,
                    V: FloatVec<N>>(pa: &P, pb: &P, pc: &P) -> (P, N) {
    let a = *pa - *pc;
    let b = *pb - *pc;

    let na = na::sqnorm(&a);
    let nb = na::sqnorm(&b);

    let dab = na::dot(&a, &b);

    let _2: N = na::cast(2.0);
    let denom = _2 * (na * nb - dab * dab);

    if denom.is_zero() {
        // the triangle is degenerate
        // FIXME: do something smarter?
        bounding_volume::point_cloud_bounding_sphere(&[pa.clone(), pb.clone(), pc.clone()])
    }
    else {
        let k = b * na - a * nb;

        let center = *pc + (a * na::dot(&k, &b) - b * na::dot(&k, &a)) / denom;
        let radius = na::dist(pa, &center);

        (center, radius)
    }
}

/// Tests if three points are exactly aligned.
pub fn is_affinely_dependent_triangle<P:  Sub<P, V>,
                                      V:  Cross<AV> + Norm<N>,
                                      AV: Norm<N>,
                                      N:  ApproxEq<N> + Float + Cast<f64>>(
                                      p1: &P,
                                      p2: &P,
                                      p3: &P)
                                      -> bool {
    let p1p2 = *p2 - *p1;
    let p1p3 = *p3 - *p1;

    // FIXME: use this as nalgebra standard epsilon?
    let _eps: N = Float::epsilon();
    let _eps_tol = _eps * na::cast(100.0f64);

    na::approx_eq_eps(&na::sqnorm(&na::cross(&p1p2, &p1p3)), &na::zero(), &(_eps_tol * _eps_tol))
}

/// Tests if a point is inside of a triangle.
pub fn is_point_in_triangle<N: Float, P: Sub<P, V>, V: FloatVec<N>>(p: &P, p1: &P, p2: &P, p3: &P) -> bool {
    let p1p2 = *p2 - *p1;
    let p2p3 = *p3 - *p2;
    let p3p1 = *p1 - *p3;

    let p1p = *p - *p1;
    let p2p = *p - *p2;
    let p3p = *p - *p3;

    let d11 = na::dot(&p1p, &p1p2);
    let d12 = na::dot(&p2p, &p2p3);
    let d13 = na::dot(&p3p, &p3p1);

    d11 >= na::zero() && d11 <= na::sqnorm(&p1p2) &&
    d12 >= na::zero() && d12 <= na::sqnorm(&p2p3) &&
    d13 >= na::zero() && d13 <= na::sqnorm(&p3p1)
}

#[cfg(test)]
mod test {
    use na;
    use na::Pnt3;

    #[test]
    fn test_triangle_area() {
        let pa = Pnt3::new(0.0f64, 5.0, 0.0);
        let pb = Pnt3::new(0.0f64, 0.0, 0.0);
        let pc = Pnt3::new(0.0f64, 0.0, 4.0);

        assert!(na::approx_eq(&super::triangle_area(&pa, &pb, &pc), &10.0));
    }
}
