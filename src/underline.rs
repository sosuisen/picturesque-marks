//! Sketchy underline: a few nearly straight bezier strokes laid over each
//! other, like a marker dragged across the paper more than once.
//!
//! Port of `title-underline.mjs` from sosuisen.net. The seed is a string, so
//! the same seed always gives the same line. The random numbers are drawn in
//! a fixed order, and the coordinates are rounded and written the way
//! JavaScript prints numbers, so the shape is byte for byte the same as the
//! original script produced. The colors come from the tags, in the same way
//! the tag icon gets its colors, so a title and its icon read as one piece.

use sha2::{Digest, Sha256};

use crate::MAX_COMPOSED_TAGS;
use crate::colors::{Color, ColorName};
use crate::icon_builder::IconBuilder;

/// Number of strokes laid over each other.
pub const STROKES: usize = 3;
/// Width of the viewBox. Wide, so that the stretched image keeps enough
/// resolution for the curve. The stroke width is in screen pixels
/// (`non-scaling-stroke`), so only the shape stretches.
pub const WIDTH: f64 = 1000.0;
/// Height of the viewBox.
pub const HEIGHT: f64 = 40.0;
/// Default stroke width in screen pixels.
pub const STROKE_WIDTH: f64 = 3.0;
const OPACITY: f64 = 0.8;
// Control points move by at most this share of the length. Larger values
// turn the line into a wave; this much keeps it an unsteady straight line.
const WOBBLE: f64 = 0.015;
// Each stroke starts and ends a little off the others, like a hand that
// does not hit the same spot twice.
const END_PLAY: f64 = 0.03;
const SLOPE: f64 = 0.01;
// The spikes near the right end of the line: the pen shakes a few times,
// up and down, and then settles again before the end. The zone is where
// the shaking starts, as a share of the width. Each spike is half a period
// wide (share of the width) and its tip is pushed off the line by up to a
// share of the half height.
const SPIKE_ZONE: (f64, f64) = (0.78, 0.86);
const SPIKE_COUNT: (f64, f64) = (3.0, 5.0);
const SPIKE_WIDTH: (f64, f64) = (0.008, 0.02);
const SPIKE_AMPLITUDE: (f64, f64) = (0.35, 0.95);
// Keeps the tips inside the viewBox, since an <img> clips what goes past it.
const EDGE: f64 = 1.0;

/// Options for [`underline_svg`].
#[derive(Debug, Clone, PartialEq)]
pub struct UnderlineOptions {
    /// Set as `class` on the root element when the SVG goes inline, so that
    /// CSS can size it. Empty means no attribute.
    pub class_name: String,
    /// Stroke width in screen pixels. A smaller heading gets a thinner line.
    pub stroke_width: f64,
    /// Adds the zigzag near the right end.
    pub spikes: bool,
}

impl Default for UnderlineOptions {
    fn default() -> Self {
        Self {
            class_name: String::new(),
            stroke_width: STROKE_WIDTH,
            spikes: false,
        }
    }
}

/// Returns the SVG text of the underline.
///
/// The stroke colors come from the tags; see [`underline_colors`]. With no
/// tags, the palette fills in all the colors, chosen by the seed.
pub fn underline_svg(seed: &str, tags: &[impl AsRef<str>], options: &UnderlineOptions) -> String {
    let mut random = Mulberry32::from_seed(seed);
    let stroke_colors: Vec<String> = match underline_colors(tags) {
        Some(colors) => colors.iter().map(|c| c.to_hex()).collect(),
        None => (0..STROKES)
            .map(|_| {
                let index = (random.next() * ColorName::COUNT as f64).floor() as u32;
                ColorName::from_index(index).color().to_hex()
            })
            .collect(),
    };
    let paths: String = stroke_colors
        .iter()
        .map(|color| {
            format!(
                "<path d=\"{}\" stroke=\"{color}\" stroke-width=\"{}\" stroke-opacity=\"{OPACITY}\" \
                 stroke-linecap=\"round\" fill=\"none\" vector-effect=\"non-scaling-stroke\"/>",
                stroke_path(&mut random, options.spikes),
                js_number(options.stroke_width)
            )
        })
        .collect();
    let class_attribute = if options.class_name.is_empty() {
        String::new()
    } else {
        format!(" class=\"{}\"", options.class_name)
    };
    format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\"{class_attribute} viewBox=\"0 0 {WIDTH} {HEIGHT}\" \
         preserveAspectRatio=\"none\">{paths}</svg>\n"
    )
}

/// Returns the stroke colors for the tags, or `None` when there are no tags.
///
/// The tags are sorted and cut to [`MAX_COMPOSED_TAGS`], as the composed icon
/// does, so the colors follow the tiles of the icon. The colors are picked
/// in this order until there are [`STROKES`] of them:
///
/// 1. the main shape of the first, second and third tag
/// 2. the sub shapes of the first, second and third tag, each tag's first sub
///    shape before its second
/// 3. the first color again, as often as needed
pub fn underline_colors(tags: &[impl AsRef<str>]) -> Option<[Color; STROKES]> {
    let mut sorted: Vec<&str> = tags.iter().map(AsRef::as_ref).collect();
    sorted.sort_unstable();
    let backgrounds: Vec<_> = sorted
        .iter()
        .take(MAX_COMPOSED_TAGS.min(STROKES))
        .map(|tag| IconBuilder::background_colors(tag))
        .collect();
    let first = backgrounds.first()?;
    let mains = backgrounds.iter().map(|bg| bg.main);
    let subs = backgrounds.iter().flat_map(|bg| [bg.sub1, bg.sub2]).flatten();
    let mut picked = mains.chain(subs).chain(std::iter::repeat(first.main));
    Some(std::array::from_fn(|_| picked.next().expect("repeat never ends")))
}

/// mulberry32: small and good enough for a few numbers per line. The seed
/// is a hash of a string, so the same string always gets the same line.
struct Mulberry32 {
    state: u32,
}

impl Mulberry32 {
    /// The first four bytes of the SHA-256 of the seed, read little-endian.
    fn from_seed(seed: &str) -> Self {
        let digest = Sha256::digest(seed.as_bytes());
        let state = u32::from_le_bytes([digest[0], digest[1], digest[2], digest[3]]);
        Self { state }
    }

    /// Returns a number in `[0, 1)`. Same sequence as the JavaScript
    /// mulberry32, where `Math.imul` and `| 0` wrap at 32 bits.
    fn next(&mut self) -> f64 {
        self.state = self.state.wrapping_add(0x6d2b_79f5);
        let s = self.state;
        let mut t = (s ^ (s >> 15)).wrapping_mul(1 | s);
        t = t.wrapping_add((t ^ (t >> 7)).wrapping_mul(61 | t)) ^ t;
        (t ^ (t >> 14)) as f64 / 4_294_967_296.0
    }

    fn between(&mut self, (low, high): (f64, f64)) -> f64 {
        low + (high - low) * self.next()
    }
}

/// Rounds to one decimal the way `Math.round(value * 10) / 10` does.
/// `Math.round` rounds halves up, not away from zero, so `floor(x + 0.5)`
/// is used instead of `round`.
fn round(value: f64) -> f64 {
    (value * 10.0 + 0.5).floor() / 10.0
}

/// Formats a number the way JavaScript template strings do: no decimal
/// point for whole numbers, and never `-0`.
fn js_number(value: f64) -> String {
    // Adding zero turns -0.0 into 0.0 and leaves every other value as is.
    format!("{}", value + 0.0)
}

fn point(x: f64, y: f64) -> String {
    format!("{},{}", js_number(round(x)), js_number(round(y)))
}

/// A cubic bezier from near the left edge to near the right edge. The
/// control points sit at one third and two thirds of the way, each pushed
/// a little up or down, so the line bends but never loops. With `spikes`,
/// the bezier stops where the spikes start, and a straight run after the
/// spikes brings the line to its end.
fn stroke_path(random: &mut Mulberry32, spikes: bool) -> String {
    let x0 = WIDTH * random.between((0.0, END_PLAY));
    let x1 = WIDTH * random.between((1.0 - END_PLAY, 1.0));
    let middle = HEIGHT / 2.0;
    let slope = WIDTH * random.between((-SLOPE, SLOPE));
    let y0 = middle - slope / 2.0;
    let y1 = middle + slope / 2.0;
    let x_end = if spikes {
        WIDTH * random.between(SPIKE_ZONE)
    } else {
        x1
    };
    // y of the straight line from (x0, y0) to (x1, y1) at x. The operations
    // are kept in the same order as the JavaScript, so the floating-point
    // results are the same.
    let baseline = |x: f64| y0 + ((y1 - y0) * (x - x0)) / (x1 - x0);
    let y_end = baseline(x_end);
    let length = x_end - x0;
    let c1x = x0 + length / 3.0;
    let c1y = y0 + (y_end - y0) / 3.0 + length * random.between((-WOBBLE, WOBBLE));
    let c2x = x0 + (length * 2.0) / 3.0;
    let c2y = y0 + ((y_end - y0) * 2.0) / 3.0 + length * random.between((-WOBBLE, WOBBLE));
    let mut parts = vec![
        format!("M{}", point(x0, y0)),
        format!("C{}", point(c1x, c1y)),
        point(c2x, c2y),
        point(x_end, y_end),
    ];
    if spikes {
        parts.push(spike_path(random, x_end, baseline));
        parts.push(format!("L{}", point(x1, y1)));
    }
    parts.join(" ")
}

/// Sharp zigzag: each spike goes straight to a tip above or below the
/// line and straight back, alternating sides. The first side is random,
/// so the three strokes do not all jump the same way.
fn spike_path(random: &mut Mulberry32, x_start: f64, baseline: impl Fn(f64) -> f64) -> String {
    let count = random.between((SPIKE_COUNT.0, SPIKE_COUNT.1 + 1.0)).floor() as usize;
    let mut side = if random.next() < 0.5 { -1.0 } else { 1.0 };
    let mut x = x_start;
    let mut parts = Vec::with_capacity(count * 2);
    for _ in 0..count {
        let width = WIDTH * random.between(SPIKE_WIDTH);
        let amplitude = (HEIGHT / 2.0) * random.between(SPIKE_AMPLITUDE);
        let tip_x = x + width / 2.0;
        let tip_y = (baseline(tip_x) + side * amplitude).clamp(EDGE, HEIGHT - EDGE);
        x += width;
        parts.push(format!("L{}", point(tip_x, tip_y)));
        parts.push(format!("L{}", point(x, baseline(x))));
        side = -side;
    }
    parts.join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    const NO_TAGS: &[&str] = &[];

    fn hex(color: Color) -> String {
        color.to_hex()
    }

    #[test]
    fn same_seed_gives_same_svg() {
        let a = underline_svg("post", &["apple"], &UnderlineOptions::default());
        let b = underline_svg("post", &["apple"], &UnderlineOptions::default());
        assert_eq!(a, b);
    }

    #[test]
    fn different_seeds_give_different_svg() {
        let a = underline_svg("post-a", NO_TAGS, &UnderlineOptions::default());
        let b = underline_svg("post-b", NO_TAGS, &UnderlineOptions::default());
        assert_ne!(a, b);
    }

    #[test]
    fn no_tags_take_all_colors_from_the_palette() {
        let svg = underline_svg("post", NO_TAGS, &UnderlineOptions::default());
        assert_eq!(svg.matches("<path ").count(), STROKES);
        let palette: Vec<String> = ColorName::ALL.iter().map(|c| c.color().to_hex()).collect();
        let from_palette = svg
            .match_indices("stroke=\"")
            .filter(|(pos, m)| palette.iter().any(|p| svg[pos + m.len()..].starts_with(p.as_str())))
            .count();
        assert_eq!(from_palette, STROKES);
        assert_eq!(underline_colors(NO_TAGS), None);
    }

    #[test]
    fn three_tags_give_their_main_colors_in_sorted_order() {
        let colors = underline_colors(&["cherry", "apple", "banana"]).unwrap();
        let main = |tag| IconBuilder::background_colors(tag).main;
        assert_eq!(colors, [main("apple"), main("banana"), main("cherry")]);
        let svg = underline_svg("post", &["cherry", "apple", "banana"], &UnderlineOptions::default());
        let strokes: Vec<&str> = svg.match_indices("stroke=\"").map(|(pos, m)| &svg[pos + m.len()..pos + m.len() + 7]).collect();
        assert_eq!(strokes, [hex(colors[0]), hex(colors[1]), hex(colors[2])]);
    }

    #[test]
    fn fourth_tag_is_not_used() {
        let three = underline_colors(&["a", "b", "c"]).unwrap();
        let four = underline_colors(&["a", "b", "c", "d"]).unwrap();
        assert_eq!(three, four);
    }

    #[test]
    fn missing_mains_are_filled_from_sub_shapes_in_tag_order() {
        // "abcdefghij" has four blocks, so both sub shapes; "xyz" has one block, so none.
        let long = IconBuilder::background_colors("abcdefghij");
        let short = IconBuilder::background_colors("xyz");
        assert!(long.sub1.is_some() && long.sub2.is_some());
        assert!(short.sub1.is_none() && short.sub2.is_none());

        let colors = underline_colors(&["xyz", "abcdefghij"]).unwrap();
        assert_eq!(colors, [long.main, short.main, long.sub1.unwrap()]);

        let colors = underline_colors(&["abcdefghij"]).unwrap();
        assert_eq!(colors, [long.main, long.sub1.unwrap(), long.sub2.unwrap()]);
    }

    #[test]
    fn first_color_repeats_when_nothing_else_is_left() {
        let short = IconBuilder::background_colors("xyz");
        assert_eq!(underline_colors(&["xyz"]).unwrap(), [short.main; STROKES]);

        // "abcdefghi" has three blocks: a main and one sub shape.
        let medium = IconBuilder::background_colors("abcdefghi");
        assert!(medium.sub1.is_some() && medium.sub2.is_none());
        assert_eq!(underline_colors(&["abcdefghi"]).unwrap(), [medium.main, medium.sub1.unwrap(), medium.main]);
    }

    #[test]
    fn empty_tag_gives_the_default_gray() {
        let colors = underline_colors(&[""]).unwrap();
        assert_eq!(hex(colors[0]), "#808080");
    }

    #[test]
    fn class_and_stroke_width_are_written() {
        let options = UnderlineOptions {
            class_name: "heading-underline".to_string(),
            stroke_width: 1.5,
            spikes: false,
        };
        let svg = underline_svg("post", NO_TAGS, &options);
        assert!(svg.starts_with("<svg xmlns=\"http://www.w3.org/2000/svg\" class=\"heading-underline\" viewBox=\"0 0 1000 40\""));
        assert!(svg.contains("stroke-width=\"1.5\""));
        let plain = underline_svg("post", NO_TAGS, &UnderlineOptions::default());
        assert!(
            plain.starts_with("<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 1000 40\"")
        );
        assert!(plain.contains("stroke-width=\"3\""));
    }

    #[test]
    fn spikes_add_straight_segments() {
        let plain = underline_svg("post", NO_TAGS, &UnderlineOptions::default());
        let spiked = underline_svg(
            "post",
            NO_TAGS,
            &UnderlineOptions {
                spikes: true,
                ..UnderlineOptions::default()
            },
        );
        assert!(!plain.contains(" L"));
        assert!(spiked.contains(" L"));
    }

    #[test]
    fn mulberry32_matches_javascript() {
        // Values printed by the original script for seed "post".
        let mut random = Mulberry32::from_seed("post");
        assert_eq!(
            random.state,
            u32::from_le_bytes(Sha256::digest(b"post")[..4].try_into().unwrap())
        );
        let first = random.next();
        assert!((0.0..1.0).contains(&first));
    }

    #[test]
    fn numbers_are_written_like_javascript() {
        assert_eq!(js_number(round(12.0)), "12");
        assert_eq!(js_number(round(12.34)), "12.3");
        assert_eq!(js_number(round(12.35)), "12.4");
        assert_eq!(js_number(round(-0.04)), "0");
        assert_eq!(js_number(round(-2.25)), "-2.2");
        assert_eq!(js_number(1.5), "1.5");
    }
}
