//! Generates a gallery HTML page from a list of fruit and vegetable tags,
//! plus a few sketchy underlines in the colors of the icons.
//!
//! The single tags are one fruit or vegetable per letter of the alphabet,
//! plus ten Japanese fruits. They are chosen so that every palette color is
//! the main color of six of them.
//!
//! Run: `cargo run --example gallery > gallery.html`

use picturesque_marks::{
    compose_ptag, generate_icon, generate_icon_from_tags, generate_ptag, generate_ptag_from_tags,
    underline_svg, ComposeLayout, ForegroundStyle, UnderlineOptions,
};

const TAGS: &[&str] = &[
    "apple",
    "banana",
    "cucumber",
    "durian",
    "elderberry",
    "feijoa",
    "grape",
    "honeydew",
    "iceberg",
    "jicama",
    "kiwi",
    "lemon",
    "mango",
    "nance",
    "onion",
    "pear",
    "quandong",
    "rambutan",
    "strawberry",
    "tangerine",
    "ugli",
    "vanilla",
    "watermelon",
    "xigua",
    "yuzu",
    "zucchini",
    "りんご",
    "林檎",
    "みかん",
    "ぶどう",
    "葡萄",
    "さくらんぼ",
    "梨",
    "かき",
    "もも",
    "石榴",
];

// Tag combinations from Cards.cs in the original repository. Used to check multi-tag composition.
const TAG_SETS: &[&[&str]] = &[
    &["フリークライミング", "card"],
    &["cardbox", "card"],
    &["cardbox", "card", "framework"],
    &["zucchini", "yuzu", "xigua", "watermelon", "vanilla"],
    &[],
];

// A few examples with a signature curve. Each entry is (signature, style, tags).
const SIGNED: &[(&str, ForegroundStyle, &[&str])] = &[
    ("Alice", ForegroundStyle::Crossing, &["apple"]),
    ("Alice", ForegroundStyle::NoCrossing, &["apple"]),
    ("Bob", ForegroundStyle::Crossing, &["apple"]),
    ("Bob", ForegroundStyle::NoCrossing, &["apple"]),
    ("Alice", ForegroundStyle::Face, &["apple"]),
    ("Bob", ForegroundStyle::Face, &["apple"]),
    ("Alice", ForegroundStyle::Crossing, &["cardbox", "card"]),
    (
        "Bob",
        ForegroundStyle::NoCrossing,
        &["cardbox", "card", "framework"],
    ),
    (
        "Carol",
        ForegroundStyle::Face,
        &["cardbox", "card", "framework"],
    ),
    ("Alice", ForegroundStyle::Crossing, &[]),
];

fn figure(svg: &str, caption: &str) {
    println!(
        "<figure style=\"margin:4px;\">{svg}<figcaption style=\"text-align:center;\">{caption}</figcaption></figure>"
    );
}

fn tags_caption(tags: &[&str]) -> String {
    if tags.is_empty() {
        "(none)".to_string()
    } else {
        tags.join(", ")
    }
}

fn main() {
    let size = 128;
    println!("<!DOCTYPE html>");
    println!("<html><head><meta charset=\"utf-8\"><title>Picturesque Marks</title></head><body>");
    println!("<h1>Picturesque Marks</h1>");

    println!("<h2>Single tag</h2>");
    println!("<div style=\"display:flex;flex-wrap:wrap;\">");
    for tag in TAGS {
        figure(&generate_ptag(tag, size, size), tag);
    }
    println!("</div>");

    for layout in [ComposeLayout::Grid, ComposeLayout::Overlay] {
        println!("<h2>Multiple tags ({layout:?})</h2>");
        println!("<div style=\"display:flex;flex-wrap:wrap;\">");
        for tags in TAG_SETS {
            figure(&compose_ptag(tags, layout, size, size), &tags_caption(tags));
        }
        println!("</div>");
    }

    println!("<h2>With signature</h2>");
    println!("<div style=\"display:flex;flex-wrap:wrap;\">");
    for (signature, style, tags) in SIGNED {
        let svg = match tags {
            [tag] => generate_icon(tag, signature, *style, size, size),
            _ => generate_icon_from_tags(tags, signature, *style, size, size),
        };
        figure(
            &svg,
            &format!("{} / sign: {signature} / {style:?}", tags_caption(tags)),
        );
    }
    println!("</div>");

    // The underline takes its colors from the same tags as the icon. The seed stands for
    // the post slug; the heading lines of one post get the slug plus a number.
    println!("<h2>Underline</h2>");
    for tags in TAG_SETS {
        let icon = generate_ptag_from_tags(tags, size, size);
        let seed = tags_caption(tags);
        let title = underline_svg(
            &seed,
            tags,
            &UnderlineOptions {
                spikes: true,
                ..UnderlineOptions::default()
            },
        );
        let heading = underline_svg(
            &format!("{seed}#h2-0"),
            tags,
            &UnderlineOptions {
                stroke_width: 1.5,
                ..UnderlineOptions::default()
            },
        );
        println!("<div style=\"display:flex;align-items:center;gap:8px;margin:4px;\">");
        println!("<div style=\"width:{size}px;flex:none;\">{icon}</div>");
        println!(
            "<div style=\"flex:1;\"><div style=\"font-size:1.5em;\">{seed}</div><div style=\"height:12px;\">{}</div>",
            title.trim_end()
        );
        println!(
            "<div style=\"font-size:1.1em;margin-top:8px;\">heading</div><div style=\"height:8px;\">{}</div></div>",
            heading.trim_end()
        );
        println!("</div>");
    }
    println!("</body></html>");
}
