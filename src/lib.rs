//! Picturesque Marks: generate SVG icons from tag strings and sketchy
//! underlines from seed strings, deterministically.
//!
//! Successor of the `picturesque-tags` crate, a Rust port of
//! <https://github.com/sosuisen/picturesque-tags>. The API is renamed:
//! every function that returns an SVG string ends in `_svg`.
//!
//! The icon background is generated from the tags. The signature mark is
//! generated from a separate string called the signature, which stands for
//! the owner's handwritten sign. An icon is the two laid over each other. The underline is generated from a seed and
//! takes its colors from an icon. The same inputs always give the same SVG.

pub mod area;
pub mod colors;
pub mod geometry;
pub mod hatch_pattern;
pub mod icon_builder;
pub mod underline;

pub use area::AreaName;
pub use colors::{Color, ColorName};
pub use geometry::{Geometry, GeometryType, ShapeType};
pub use hatch_pattern::HatchPatternType;
pub use icon_builder::{BackgroundColors, IconBuilder, SignatureStyle, background_colors};
pub use underline::{UnderlineOptions, underline_colors, underline_svg};

/// Maximum number of tags in a composed icon.
/// Tiles go to the top-left, top-right, bottom-left and bottom-right.
pub const MAX_COMPOSED_TAGS: usize = 4;

/// How several tags are composed into one icon.
///
/// More layouts will be added, so match on it with a wildcard arm.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ComposeLayout {
    /// Up to four tags as W/2 x H/2 tiles in a 2 x 2 grid, in the given
    /// order: top-left, top-right, bottom-left, bottom-right.
    Grid,
    /// The first tag at full size, and up to three more tags as W/2 x H/2
    /// tiles laid over it at 50% opacity. One more tag goes bottom-right.
    /// Two go top-right and bottom-left. Three go top-right, bottom-left
    /// and bottom-right, in the given order. This is the default.
    #[default]
    Overlay,
}

/// Opacity of the tiles laid over the first tag in [`ComposeLayout::Overlay`].
const OVERLAY_OPACITY: f64 = 0.5;

/// The signature string and how to draw it.
#[derive(Debug, Clone, Copy)]
struct Signature<'a> {
    text: &'a str,
    style: SignatureStyle,
    color: Color,
}

/// Generates the SVG of the signature alone: the mark drawn from
/// `signature` in the `style` and `color`, with no background.
///
/// The color is the stroke of the curve and the eyes, or the fill of
/// `SignatureStyle::Blob`. `Color::BLACK` is the usual choice.
pub fn signature_svg(
    signature: &str,
    style: SignatureStyle,
    color: Color,
    width: i32,
    height: i32,
) -> String {
    let mut builder = IconBuilder::new(width, height);
    builder.add_signature(signature, style, color);
    builder.to_svg()
}

/// Generates the SVG of the background alone: the shapes drawn from the tags.
///
/// - No tags: the default gray circle.
/// - One tag: the shapes of that tag.
/// - Two or more tags: up to [`MAX_COMPOSED_TAGS`] in the given order,
///   composed in the `layout`. The order is kept because the caller puts
///   the main tag first.
pub fn background_svg(
    tags: &[impl AsRef<str>],
    layout: ComposeLayout,
    width: i32,
    height: i32,
) -> String {
    from_tags(tags, layout, width, height, None)
}

/// Generates the SVG of an icon: the background of [`background_svg`] with
/// the signature of [`signature_svg`] drawn once, large, over the whole icon.
pub fn icon_svg(
    tags: &[impl AsRef<str>],
    layout: ComposeLayout,
    signature: &str,
    style: SignatureStyle,
    color: Color,
    width: i32,
    height: i32,
) -> String {
    from_tags(
        tags,
        layout,
        width,
        height,
        Some(Signature {
            text: signature,
            style,
            color,
        }),
    )
}

fn from_tags(
    tags: &[impl AsRef<str>],
    layout: ComposeLayout,
    width: i32,
    height: i32,
    signature: Option<Signature<'_>>,
) -> String {
    match tags {
        [] => build_icon("", width, height, signature).to_svg(),
        [tag] => build_icon(tag.as_ref(), width, height, signature).to_svg(),
        _ => match layout {
            ComposeLayout::Grid => compose_grid(tags, width, height, signature),
            ComposeLayout::Overlay => compose_overlay(tags, width, height, signature),
        },
    }
}

/// Composes several tags into one icon in the [`ComposeLayout::Grid`] layout.
///
/// Lays out up to four of the tags, in the given order, as W/2 x H/2 tiles
/// in a 2 x 2 grid. The first tag is the top-left tile.
/// Each tile is embedded as a nested `<svg>`, so shapes outside a tile are clipped.
/// The signature curve, if any, is drawn once over the whole icon, not per tile.
fn compose_grid(
    tags: &[impl AsRef<str>],
    width: i32,
    height: i32,
    signature: Option<Signature<'_>>,
) -> String {
    let tile_width = width / 2;
    let tile_height = height / 2;
    let mut layers: Vec<String> = tags
        .iter()
        .map(AsRef::as_ref)
        .take(MAX_COMPOSED_TAGS)
        .enumerate()
        .map(|(i, tag)| {
            // 0: top-left, 1: top-right, 2: bottom-left, 3: bottom-right
            let x = (i % 2) as i32 * tile_width;
            let y = (i / 2) as i32 * tile_height;
            let tile = build_icon(tag, tile_width, tile_height, None).to_svg();
            nest_svg(&tile, x, y)
        })
        .collect();

    if let Some(sig) = signature {
        let mut layer = IconBuilder::new(width, height);
        layer.add_signature(sig.text, sig.style, sig.color);
        layers.push(nest_svg(&layer.to_svg(), 0, 0));
    }

    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" viewBox="0 0 {width} {height}">{}</svg>"#,
        layers.concat()
    )
}

/// Composes several tags into one icon in the [`ComposeLayout::Overlay`] layout.
///
/// The first tag fills the whole icon. The next tags, up to three, are
/// W/2 x H/2 tiles drawn over it at [`OVERLAY_OPACITY`]. The tiles sit in
/// the grid cells that are not top-left, chosen so that one extra tag goes
/// bottom-right and two extra tags go to opposite corners.
/// The signature curve, if any, is drawn once on top, fully opaque.
fn compose_overlay(
    tags: &[impl AsRef<str>],
    width: i32,
    height: i32,
    signature: Option<Signature<'_>>,
) -> String {
    let tile_width = width / 2;
    let tile_height = height / 2;
    let tags: Vec<&str> = tags
        .iter()
        .map(AsRef::as_ref)
        .take(MAX_COMPOSED_TAGS)
        .collect();
    let (first, rest) = tags.split_first().expect("two or more tags");

    // Cells as (column, row). The first tag is not a tile, so top-left is free.
    let cells: &[(i32, i32)] = match rest.len() {
        1 => &[(1, 1)],
        2 => &[(1, 0), (0, 1)],
        _ => &[(1, 0), (0, 1), (1, 1)],
    };

    let mut layers = vec![nest_svg(
        &build_icon(first, width, height, None).to_svg(),
        0,
        0,
    )];
    for (tag, &(column, row)) in rest.iter().zip(cells) {
        let tile = build_icon(tag, tile_width, tile_height, None).to_svg();
        let tile = nest_svg(&tile, column * tile_width, row * tile_height);
        layers.push(format!(r#"<g opacity="{OVERLAY_OPACITY}">{tile}</g>"#));
    }

    if let Some(sig) = signature {
        let mut layer = IconBuilder::new(width, height);
        layer.add_signature(sig.text, sig.style, sig.color);
        layers.push(nest_svg(&layer.to_svg(), 0, 0));
    }

    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" viewBox="0 0 {width} {height}">{}</svg>"#,
        layers.concat()
    )
}

/// Turns a standalone SVG string into a nested `<svg>` placed at `(x, y)`.
fn nest_svg(svg: &str, x: i32, y: i32) -> String {
    svg.replacen("<svg ", &format!(r#"<svg x="{x}" y="{y}" "#), 1)
}

fn build_icon(tag: &str, width: i32, height: i32, signature: Option<Signature<'_>>) -> IconBuilder {
    let mut builder = IconBuilder::new(width, height);
    builder.add_background_from_tag(tag);
    if let Some(sig) = signature {
        builder.add_signature(sig.text, sig.style, sig.color);
    }
    builder
}

#[cfg(test)]
mod tests {
    use super::*;

    const SIGN: &str = "Hidekazu";
    const STYLE: SignatureStyle = SignatureStyle::Crossing;

    #[test]
    fn default_layout_is_overlay() {
        assert_eq!(ComposeLayout::default(), ComposeLayout::Overlay);
    }

    /// Positions of the nested `<svg>` tiles, in document order.
    fn tile_positions(svg: &str) -> Vec<(i32, i32)> {
        svg.match_indices("<svg x=\"")
            .map(|(pos, _)| {
                let mut parts = svg[pos..].split('"');
                let x = parts.nth(1).unwrap().parse().unwrap();
                let y = parts.nth(1).unwrap().parse().unwrap();
                (x, y)
            })
            .collect()
    }

    #[test]
    fn overlay_puts_the_first_tag_at_full_size() {
        let svg = background_svg(&["apple", "banana"], ComposeLayout::Overlay, 100, 80);
        let first = background_svg(&["apple"], ComposeLayout::default(), 100, 80);
        let first_body = first.split_once('>').map(|(_, rest)| rest).unwrap();
        assert!(svg.contains(
            r#"<svg x="0" y="0" xmlns="http://www.w3.org/2000/svg" width="100" height="80""#
        ));
        assert!(svg.contains(first_body));
    }

    #[test]
    fn overlay_places_the_other_tags_by_their_count() {
        let two = background_svg(&["a", "b"], ComposeLayout::Overlay, 100, 80);
        assert_eq!(tile_positions(&two), vec![(0, 0), (50, 40)]);
        let three = background_svg(&["a", "b", "c"], ComposeLayout::Overlay, 100, 80);
        assert_eq!(tile_positions(&three), vec![(0, 0), (50, 0), (0, 40)]);
        let four = background_svg(&["a", "b", "c", "d"], ComposeLayout::Overlay, 100, 80);
        assert_eq!(
            tile_positions(&four),
            vec![(0, 0), (50, 0), (0, 40), (50, 40)]
        );
        // The fifth tag is dropped, as in the grid.
        let five = background_svg(&["a", "b", "c", "d", "e"], ComposeLayout::Overlay, 100, 80);
        assert_eq!(four, five);
    }

    #[test]
    fn overlay_tiles_are_half_transparent_and_the_curve_is_not() {
        let svg = icon_svg(
            &["a", "b", "c"],
            ComposeLayout::Overlay,
            SIGN,
            STYLE,
            Color::BLACK,
            100,
            100,
        );
        assert_eq!(svg.matches(r#"<g opacity="0.5">"#).count(), 2);
        // The curve layer comes after the tiles and outside any opacity group.
        let curve = svg.rfind("<svg x=\"0\" y=\"0\"").unwrap();
        assert!(svg[curve..].contains("<path "));
        assert!(!svg[curve..].contains("opacity"));
    }

    #[test]
    fn blob_matches_the_face_outline() {
        let cyan = Color::rgb(0x5F, 0xD3, 0xD3);
        let filled = signature_svg(SIGN, SignatureStyle::Blob, cyan, 100, 100);
        let face = signature_svg(SIGN, SignatureStyle::Face, Color::BLACK, 100, 100);
        let path_d = |svg: &str| {
            svg.split("d=\"")
                .nth(1)
                .unwrap()
                .split('"')
                .next()
                .unwrap()
                .to_string()
        };
        assert_eq!(path_d(&filled), path_d(&face));
        assert!(filled.contains(r##"fill="#5FD3D3"/>"##));
        assert!(!filled.contains("<line "));
        assert_eq!(
            filled,
            signature_svg(SIGN, SignatureStyle::Blob, cyan, 100, 100)
        );
    }

    #[test]
    fn single_tag_is_same_as_the_builder() {
        let from_tags = background_svg(&["apple"], ComposeLayout::default(), 100, 100);
        let mut builder = IconBuilder::new(100, 100);
        builder.add_background_from_tag("apple");
        assert_eq!(from_tags, builder.to_svg());
    }

    #[test]
    fn icon_svg_is_deterministic() {
        let a = icon_svg(
            &["apple"],
            ComposeLayout::default(),
            SIGN,
            STYLE,
            Color::BLACK,
            100,
            100,
        );
        let b = icon_svg(
            &["apple"],
            ComposeLayout::default(),
            SIGN,
            STYLE,
            Color::BLACK,
            100,
            100,
        );
        assert_eq!(a, b);
        assert!(a.contains("<path d=\"M "));
    }

    #[test]
    fn signature_changes_only_the_curve() {
        let a = icon_svg(
            &["apple"],
            ComposeLayout::default(),
            "alice",
            STYLE,
            Color::BLACK,
            100,
            100,
        );
        let b = icon_svg(
            &["apple"],
            ComposeLayout::default(),
            "bob",
            STYLE,
            Color::BLACK,
            100,
            100,
        );
        let background = |svg: &str| svg.split("<path ").next().unwrap().to_string();
        assert_eq!(background(&a), background(&b));
        assert_ne!(a, b);
    }

    #[test]
    fn style_changes_only_the_curve() {
        let a = icon_svg(
            &["apple"],
            ComposeLayout::default(),
            SIGN,
            SignatureStyle::Crossing,
            Color::BLACK,
            100,
            100,
        );
        let b = icon_svg(
            &["apple"],
            ComposeLayout::default(),
            SIGN,
            SignatureStyle::NoCrossing,
            Color::BLACK,
            100,
            100,
        );
        let background = |svg: &str| svg.split("<path ").next().unwrap().to_string();
        assert_eq!(background(&a), background(&b));
        assert_ne!(a, b);
    }

    #[test]
    fn no_tags_gives_default_icon() {
        let svg = icon_svg(
            &[] as &[&str],
            ComposeLayout::default(),
            SIGN,
            STYLE,
            Color::BLACK,
            100,
            100,
        );
        assert!(svg.contains(r##"fill="#808080""##));
        assert!(svg.contains("<path d=\"M "));
    }

    #[test]
    fn multiple_tags_are_tiled_in_the_given_order() {
        let svg = background_svg(&["apple", "banana"], ComposeLayout::Grid, 100, 80);
        assert!(svg.starts_with(
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="80" viewBox="0 0 100 80">"#
        ));
        // The first tag is top-left and the second is top-right.
        let apple_tile = svg.find(r#"<svg x="0" y="0" "#).expect("top-left tile");
        let banana_tile = svg.find(r#"<svg x="50" y="0" "#).expect("top-right tile");
        assert!(apple_tile < banana_tile);
        let apple_body = background_svg(&["apple"], ComposeLayout::default(), 50, 40);
        let apple_body = apple_body.split_once('>').map(|(_, rest)| rest).unwrap();
        assert!(svg[apple_tile..banana_tile].contains(apple_body));
    }

    #[test]
    fn three_tags_leave_bottom_right_empty() {
        let svg = background_svg(&["c", "a", "b"], ComposeLayout::Grid, 100, 100);
        assert!(svg.contains(r#"<svg x="0" y="50" "#));
        assert!(!svg.contains(r#"<svg x="50" y="50" "#));
    }

    #[test]
    fn more_than_four_tags_are_truncated() {
        let svg = background_svg(&["e", "d", "c", "b", "a"], ComposeLayout::Grid, 100, 100);
        assert_eq!(svg.matches("<svg x=").count(), MAX_COMPOSED_TAGS);
    }

    #[test]
    fn composed_icon_draws_one_large_curve_on_top() {
        let svg = icon_svg(
            &["b", "a", "c", "d"],
            ComposeLayout::Grid,
            SIGN,
            STYLE,
            Color::BLACK,
            100,
            100,
        );
        // Four tiles plus one full-size signature layer.
        assert_eq!(svg.matches("<svg x=").count(), MAX_COMPOSED_TAGS + 1);
        assert_eq!(svg.matches("<path ").count(), 1);
        let curve_layer = svg.rfind("<svg x=").unwrap();
        assert!(svg[curve_layer..].starts_with(
            r#"<svg x="0" y="0" xmlns="http://www.w3.org/2000/svg" width="100" height="100""#
        ));
        assert!(svg[curve_layer..].contains("<path "));
        // The curve is the same one a single icon of the same size would get.
        let mut expected = IconBuilder::new(100, 100);
        expected.add_signature(SIGN, STYLE, Color::BLACK);
        let expected_path = expected.to_svg();
        let expected_path = expected_path.split("<path ").nth(1).unwrap();
        assert!(svg.contains(expected_path));
    }

    #[test]
    fn composed_pattern_ids_name_the_same_definition() {
        // Tiles may repeat a pattern id. That is harmless only when every
        // definition with the same id is the same text.
        let svg = background_svg(
            &["abcdef", "ghijkl", "mnopqr", "stuvwx"],
            ComposeLayout::default(),
            200,
            200,
        );
        let mut by_id: std::collections::HashMap<&str, &str> = std::collections::HashMap::new();
        for (pos, _) in svg.match_indices("<pattern id=\"") {
            let end = svg[pos..].find("</pattern>").unwrap() + pos;
            let def = &svg[pos..end];
            let id = def.split('"').nth(1).unwrap();
            assert!(id.starts_with("pat-"), "{id}");
            if let Some(seen) = by_id.insert(id, def) {
                assert_eq!(seen, def, "{id}");
            }
        }
        assert!(!by_id.is_empty());
        for id in by_id.keys() {
            assert!(svg.contains(&format!("url(#{id})")));
        }
    }
}
