# picturesque-marks

Successor of [picturesque-tags](https://github.com/sosuisen/picturesque-tags-rust).
It generates two kinds of SVG marks, and the same inputs always produce the same SVG.

- **Tag icons**, as in picturesque-tags. The background comes from the tag.
  The hand-drawn style mark on top comes from a separate string called the signature,
  which stands for the owner's handwritten sign. The mark alone is also available.
- **Sketchy underlines**: a few nearly straight bezier strokes laid over each other,
  like a marker dragged across the paper more than once. The shape comes from a seed string.
  The colors come from the tags, the same way the tag icon gets its colors,
  so a title and its icon read as one piece.

Every function that returns an SVG string ends in `_svg`.

## Usage

### Tag icons

```rust
use picturesque_marks::{Color, ComposeLayout, SignatureStyle, background_svg, icon_svg, signature_svg};

// Three kinds of SVG: the signature mark, the background, and the icon that lays the two over each other.

// The signature mark alone, generated from the signature string.
// SignatureStyle::Crossing draws a curve that crosses itself exactly once, like a loop in a sign.
// SignatureStyle::NoCrossing draws a curve that never crosses itself.
// SignatureStyle::Face is NoCrossing plus two short vertical bars as eyes.
// SignatureStyle::Blob is the Face outline filled, without stroke or eyes.
// The color is the stroke of the curve and the eyes, or the fill of Blob.
let svg = signature_svg("Hidekazu", SignatureStyle::Crossing, Color::BLACK, 128, 128);
let svg = signature_svg("Hidekazu", SignatureStyle::Blob, Color::rgb(0xF9, 0xA2, 0xA2), 128, 128);

// The background alone, generated from the tags.
let svg = background_svg(&["apple"], ComposeLayout::default(), 100, 100);

// The icon: the background with the signature mark on top.
let svg = icon_svg(&["apple"], ComposeLayout::default(), "Hidekazu", SignatureStyle::Crossing, Color::BLACK, 128, 128);

// Multiple tags: up to four are composed into one icon, in the given order.
// The layout is named by `ComposeLayout`. Put the main tag first in both layouts.
// - `Overlay` (default): the first tag at full size, the others as half-transparent tiles over it.
//   One more tag goes bottom-right. Two go top-right and bottom-left.
//   Three go top-right, bottom-left and bottom-right.
// - `Grid`: a 2x2 grid of tiles. The first tag is the top-left tile.
// The signature mark is drawn once over the whole icon.
let svg = icon_svg(&["cardbox", "card"], ComposeLayout::Overlay, "Hidekazu", SignatureStyle::NoCrossing, Color::BLACK, 128, 128);
let svg = icon_svg(&["cardbox", "card"], ComposeLayout::Grid, "Hidekazu", SignatureStyle::NoCrossing, Color::BLACK, 128, 128);
```

Use `IconBuilder` directly for finer control.

```rust
use picturesque_marks::{Color, HatchPatternType, IconBuilder, SignatureStyle};

let mut builder = IconBuilder::new(100, 100);
builder
    .add_background_from_tag("apple")
    .add_circle(HatchPatternType::Dots, Color::rgb(0x9B, 0xA9, 0xEB), 20.0, 0.0, 0.0)
    .add_signature("Hidekazu", SignatureStyle::Crossing, Color::BLACK);
let svg = builder.to_svg();
```

### Underlines

```rust
use picturesque_marks::{UnderlineOptions, underline_colors, underline_svg};

// The line under a post title, in the colors of the post's tags.
// The slug seeds the shape, so a rebuild draws the same line.
// `spikes` makes the pen shake a few times near the right end.
let tags = ["cardbox", "card"];
let title = underline_svg("my-post", &tags, &UnderlineOptions { spikes: true, ..UnderlineOptions::default() });

// A thinner line under each h2, with a class so that CSS can size it when it goes inline.
let heading = underline_svg(
    "my-post#h2-0",
    &tags,
    &UnderlineOptions { class_name: "heading-underline".to_string(), stroke_width: 1.5, spikes: false },
);

// The three stroke colors alone, as `Color` values. `None` when there are no tags.
let colors = underline_colors(&tags);
```

The SVG has `viewBox="0 0 1000 40"` and `preserveAspectRatio="none"`, so it stretches to the width it is given.
The stroke width is in screen pixels (`vector-effect="non-scaling-stroke"`), so only the shape stretches.

Three strokes are drawn. Their colors are picked from the tags in this order, until there are three:

1. The main shape color of the first, second and third tag.
2. The sub shape colors of the first, second and third tag. Each tag's first sub shape comes before its second.
3. The first color again, as often as needed.

The tags are cut to four in the given order, as the composed icon does, so the colors follow the tiles of the icon.
`background_colors(tag)` gives the main and sub shape colors of one tag.
With no tags, all three colors come from the palette, chosen by the seed.

### Command line

```sh
cargo run -- --size 128 apple > apple.svg
cargo run -- --size 128x96 --signature Hidekazu apple > apple.svg
cargo run -- --size 128 --signature Hidekazu --style no-crossing apple > apple.svg
cargo run -- --size 128 --signature Hidekazu --style face apple > apple.svg
cargo run -- --size 128 --signature Hidekazu --style blob --color F9A2A2 apple > apple.svg
# The signature mark alone
cargo run -- signature --size 128 --style blob --color F9A2A2 Hidekazu > hidekazu.svg
# Up to four tags are composed into one icon, in the given order. --layout is overlay (default) or grid
cargo run -- --size 128 --signature Hidekazu --layout grid cardbox card > cardbox_card.svg
cargo run -- --size 128 --signature Hidekazu --layout overlay cardbox card > cardbox_card_overlay.svg
# Underlines: the seed comes first, then the tags
cargo run -- underline --spikes my-post cardbox card > my-post_underline.svg
cargo run -- underline --class heading-underline --stroke-width 1.5 "my-post#h2-0" cardbox card > h2.svg
```

### Gallery

Generates an HTML page with the same tag list as the original Blazor demo, plus a few underlines.

```sh
cargo run --example gallery > gallery.html
```

### Swatches

Generates an HTML page that shows every palette color crossed with every hatch pattern, one matrix per shape. Use it to tune the colors and the patterns by eye.

```sh
cargo run --example swatches > swatches.html
```

## Tests

```sh
cargo test
```

`tests/golden.rs` holds SVGs produced by the original JavaScript `title-underline.mjs` of sosuisen.net.
The Rust port must match their shape byte for byte, so that a re-sync of the site does not redraw every line.
The original took colors instead of tags, so the test swaps its placeholder colors for the ones picked from the tags.

## How the underline is drawn

The seed string is hashed with SHA-256. The first four bytes, read little-endian, start a mulberry32 generator.
The random numbers are drawn in a fixed order: the palette colors when there are no tags, then each stroke.
Within a stroke: both ends, the slope, the spike start, the two control points, then the spikes.
Changing this order changes every line, even with the same seed.

Each stroke is one cubic bezier. Its ends sit within 3% of the edges, the slope is within ±1% of the width,
and the control points at one third and two thirds are pushed off the straight line by at most 1.5% of the length.
With `spikes`, the bezier stops at 78% to 86% of the width, three to five sharp zigzags follow,
and a straight run brings the line to its end. Spike tips stay inside the viewBox.
Coordinates are rounded to one decimal and written the way JavaScript prints numbers.

## Changes from 0.2.0

Version 0.3.0 renames most of the API. With `Color::BLACK` as the signature color,
the SVG output is the same as 0.2.0 for the same inputs.

The three kinds of SVG are now named after what they draw:

| 0.3.0 | 0.2.0 |
|---|---|
| `signature_svg(signature, style, color, w, h)` | none; the signature could not be drawn alone |
| `background_svg(tags, layout, w, h)` | `generate_ptag`, `generate_ptag_from_tags`, `compose_ptag` |
| `icon_svg(tags, layout, signature, style, color, w, h)` | `generate_icon`, `generate_icon_from_tags`, `compose_icon` |
| `background_colors(tag)` | `IconBuilder::background_colors(tag)` |
| `underline_svg`, `underline_colors` | unchanged |

Breaking changes:

- Every function that returns an SVG string ends in `_svg`. Tags are always a slice, so one tag is `&["apple"]`.
  The layout is always given; pass `ComposeLayout::default()` for `Overlay`.
- `ForegroundStyle` is now `SignatureStyle`. It has a new style, `Blob`: the `Face` outline filled, with no stroke and no eyes.
- The signature takes a `Color`. It is the stroke of the curve and the eyes, or the fill of `Blob`.
  `signature_svg`, `icon_svg`, `IconBuilder::add_signature` and `IconBuilder::add_default_icon` take it. Use `Color::BLACK` for the old look.
- `IconBuilder` methods: `generate_bg_from_text` is `add_background_from_tag`, `add_foreground` is `add_signature`,
  `default_background` is `add_default_background`, `default_icon` is `add_default_icon`.
- `Color::change_brightness` is `Color::with_brightness`. `Geometry::generate_from_hash` is `Geometry::from_hash`.

Added:

- `Color::BLACK`.
- Command line: `--style blob`, `--color HEX` for the signature color, and the `signature` subcommand that draws the signature alone.

## Differences from the original C# picturesque-tags

- The signature mark is generated from the signature with a seeded PCG32 generator. The original used a non-deterministic random generator.
- The SVG is built as a string without an external SVG library.
- Pattern IDs are sequential numbers instead of GUIDs, so the same tag yields the same SVG string.
- Colors are always written as `#RRGGBB`.
- Characters outside the BMP count as one character (Rust `char`). The original counts them as two UTF-16 units.
