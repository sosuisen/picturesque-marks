//! Petal outlines for `BackgroundStyle::Petal`: one asymmetric petal, and
//! the set of three copies the background draws. See ADR-007.
//!
//! The set is not symmetric on purpose. It starts from one petal whose two
//! edges differ, and lays two copies of it under the first: one fanned out
//! from the tip, one crossing the first like an X.

use rand::{RngExt, SeedableRng};
use rand_pcg::Pcg32;

/// A closed outline as points. The coordinates of a [`PetalSet`] are
/// multiples of the icon width, centered at the icon center.
pub type Outline = Vec<(f32, f32)>;

/// The three petals of one tag.
#[derive(Debug, Clone, PartialEq)]
pub struct PetalSet {
    /// The petal drawn on top, in the main color.
    pub main: Outline,
    /// A copy of `main` fanned out from the tip, drawn under it.
    pub sub1: Outline,
    /// A copy of `main` rotated the other way around its middle, so it
    /// crosses `main` like an X. Drawn at the bottom.
    pub sub2: Outline,
}

/// Points sampled on each edge of a petal. Enough to look smooth at 128px
/// without making the SVG large.
const SAMPLES_PER_EDGE: usize = 40;

/// Fraction of the icon width that the whole set spans.
const FILL: f64 = 0.9;

// Ranges of the random values. The width and the bulge are drawn once per
// edge, which is what makes the petal asymmetric. The width, the sharpness,
// the bend and the tilt are bounded below or above so the petal does not
// turn into a thin claw or lie flat. A hatched thin petal is hard to read.
const EDGE_WIDTH: std::ops::Range<f64> = 0.45..0.70;
const EDGE_BULGE: std::ops::Range<f64> = -0.2..0.4;
const TIP_SHARPNESS: std::ops::Range<f64> = 0.3..0.8;
const BEND: std::ops::Range<f64> = -0.25..0.25;
const BASE_HALF_WIDTH: std::ops::Range<f64> = 0.0..0.2;
const TILT_DEGREES: std::ops::Range<f64> = -35.0..35.0;
const FAN_DEGREES: std::ops::Range<f64> = 18.0..40.0;
const FAN_SHIFT_X: std::ops::Range<f64> = 0.10..0.22;
const FAN_SHIFT_Y: std::ops::Range<f64> = 0.04..0.14;
const CROSS_DEGREES: std::ops::Range<f64> = 35.0..55.0;

/// The tip of the base petal. The base of the petal is at (0, 1).
const TIP: (f64, f64) = (0.0, -1.0);
/// The pivot of the crossing petal. A little below the middle, so the two
/// petals cross where they are widest.
const CROSS_PIVOT: (f64, f64) = (0.0, 0.1);

/// Builds the set for a seed. The same seed always gives the same set.
pub fn petal_set(seed: u64) -> PetalSet {
    // PCG32 is used instead of rand's default generators because its algorithm
    // is fixed, so the same tag keeps giving the same petals across versions.
    let mut rng = Pcg32::seed_from_u64(seed);
    let base = base_petal(&mut rng);
    let tilt = rng.random_range(TILT_DEGREES);

    // The fan and the cross go to opposite sides, so the three tips spread out.
    let side = if rng.random_range(0..2) == 0 {
        1.0
    } else {
        -1.0
    };
    let fan = rotate(&base, side * rng.random_range(FAN_DEGREES), TIP);
    let shift = (
        side * rng.random_range(FAN_SHIFT_X),
        rng.random_range(FAN_SHIFT_Y),
    );
    let fan: Vec<_> = fan
        .iter()
        .map(|&(x, y)| (x + shift.0, y + shift.1))
        .collect();
    let cross = rotate(&base, -side * rng.random_range(CROSS_DEGREES), CROSS_PIVOT);

    let tilted = [base, fan, cross].map(|petal| rotate(&petal, tilt, (0.0, 0.0)));
    let [main, sub1, sub2] = fit(tilted);
    PetalSet { main, sub1, sub2 }
}

/// One petal with its tip at [`TIP`] and its base at (0, 1).
///
/// Each edge is one cubic Bezier from the tip to the base. The right edge
/// and the left edge draw their own width and bulge, so they differ.
/// The axis is then bent sideways, most at the middle and not at the ends.
fn base_petal(rng: &mut Pcg32) -> Vec<(f64, f64)> {
    let width_right = rng.random_range(EDGE_WIDTH);
    let width_left = rng.random_range(EDGE_WIDTH);
    let bulge_right = rng.random_range(EDGE_BULGE);
    let bulge_left = rng.random_range(EDGE_BULGE);
    let sharpness = rng.random_range(TIP_SHARPNESS);
    let bend = rng.random_range(BEND);
    let base_half_width = rng.random_range(BASE_HALF_WIDTH);

    let edge = |width: f64, bulge: f64| {
        [
            (width * sharpness, -1.0 + (1.0 + bulge) * 0.6),
            (width * 1.15, bulge + 0.35),
        ]
    };
    let [r1, r2] = edge(width_right, bulge_right);
    let [l1, l2] = edge(width_left, bulge_left);

    let mut points = bezier(TIP, r1, r2, (base_half_width, 1.0));
    let left = bezier((-base_half_width, 1.0), (-l2.0, l2.1), (-l1.0, l1.1), TIP);
    points.extend(left);

    points
        .into_iter()
        .map(|(x, y)| (x + bend * (1.0 - y * y), y))
        .collect()
}

/// Samples a cubic Bezier from `p0` to `p3`, without the end point. The
/// end point is the start of the next edge, so it is not repeated.
fn bezier(p0: (f64, f64), p1: (f64, f64), p2: (f64, f64), p3: (f64, f64)) -> Vec<(f64, f64)> {
    (0..SAMPLES_PER_EDGE)
        .map(|i| {
            let t = i as f64 / SAMPLES_PER_EDGE as f64;
            let u = 1.0 - t;
            let w = [u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t];
            (
                w[0] * p0.0 + w[1] * p1.0 + w[2] * p2.0 + w[3] * p3.0,
                w[0] * p0.1 + w[1] * p1.1 + w[2] * p2.1 + w[3] * p3.1,
            )
        })
        .collect()
}

fn rotate(points: &[(f64, f64)], degrees: f64, pivot: (f64, f64)) -> Vec<(f64, f64)> {
    let (sin, cos) = degrees.to_radians().sin_cos();
    points
        .iter()
        .map(|&(x, y)| {
            let (dx, dy) = (x - pivot.0, y - pivot.1);
            (pivot.0 + dx * cos - dy * sin, pivot.1 + dx * sin + dy * cos)
        })
        .collect()
}

/// Scales and moves the petals together so that their bounding box spans
/// [`FILL`] of the icon width and is centered at the origin.
///
/// All three are fitted together, even when the tag draws fewer. That keeps
/// the main petal in the same place whether or not the sub petals are drawn.
fn fit(petals: [Vec<(f64, f64)>; 3]) -> [Outline; 3] {
    let all = petals.iter().flatten();
    let (min_x, max_x, min_y, max_y) = all.fold(
        (f64::MAX, f64::MIN, f64::MAX, f64::MIN),
        |(min_x, max_x, min_y, max_y), &(x, y)| {
            (min_x.min(x), max_x.max(x), min_y.min(y), max_y.max(y))
        },
    );
    let scale = FILL / (max_x - min_x).max(max_y - min_y);
    let center = ((min_x + max_x) / 2.0, (min_y + max_y) / 2.0);
    petals.map(|petal| {
        petal
            .into_iter()
            .map(|(x, y)| {
                (
                    ((x - center.0) * scale) as f32,
                    ((y - center.1) * scale) as f32,
                )
            })
            .collect()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base_petal_starts_at_the_tip_and_has_both_edges() {
        let mut rng = Pcg32::seed_from_u64(1);
        let petal = base_petal(&mut rng);
        assert_eq!(petal.len(), 2 * SAMPLES_PER_EDGE);
        assert_eq!(petal[0], TIP);
        // The bend does not move the tip or the base, so y stays in [-1, 1].
        assert!(petal.iter().all(|&(_, y)| (-1.0..=1.0).contains(&y)));
        // The right edge comes first and leans right; the left edge leans left.
        assert!(petal[SAMPLES_PER_EDGE / 2].0 > 0.0);
        assert!(petal[SAMPLES_PER_EDGE + SAMPLES_PER_EDGE / 2].0 < 0.0);
    }

    #[test]
    fn base_petal_is_asymmetric() {
        let mut rng = Pcg32::seed_from_u64(1);
        let petal = base_petal(&mut rng);
        let right = petal[SAMPLES_PER_EDGE / 2];
        // The mirror of the right edge point is not on the left edge.
        let mirrored = (-right.0, right.1);
        let on_left = petal[SAMPLES_PER_EDGE..]
            .iter()
            .any(|&(x, y)| (x - mirrored.0).abs() < 1e-3 && (y - mirrored.1).abs() < 1e-3);
        assert!(!on_left);
    }

    #[test]
    fn set_is_deterministic_and_depends_on_the_seed() {
        assert_eq!(petal_set(7), petal_set(7));
        assert_ne!(petal_set(7), petal_set(8));
    }

    #[test]
    fn set_fits_the_icon() {
        let set = petal_set(3);
        let all = [&set.main, &set.sub1, &set.sub2];
        let xs = all.iter().flat_map(|p| p.iter().map(|&(x, _)| x));
        let ys = all.iter().flat_map(|p| p.iter().map(|&(_, y)| y));
        let (min_x, max_x) = xs.fold((f32::MAX, f32::MIN), |(a, b), x| (a.min(x), b.max(x)));
        let (min_y, max_y) = ys.fold((f32::MAX, f32::MIN), |(a, b), y| (a.min(y), b.max(y)));
        let span = (max_x - min_x).max(max_y - min_y);
        assert!((span - FILL as f32).abs() < 1e-4);
        assert!((min_x + max_x).abs() < 1e-4);
        assert!((min_y + max_y).abs() < 1e-4);
    }

    #[test]
    fn sub_petals_are_the_main_petal_moved() {
        // A rigid move keeps the distance between the first two points.
        let set = petal_set(5);
        let dist = |p: &Outline| ((p[0].0 - p[10].0).powi(2) + (p[0].1 - p[10].1).powi(2)).sqrt();
        assert!((dist(&set.main) - dist(&set.sub1)).abs() < 1e-4);
        assert!((dist(&set.main) - dist(&set.sub2)).abs() < 1e-4);
        assert_ne!(set.main, set.sub1);
        assert_ne!(set.main, set.sub2);
    }
}
