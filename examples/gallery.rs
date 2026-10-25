//! Generates a gallery HTML page from a list of fruit and vegetable tags,
//! plus a few sketchy underlines in the colors of the icons.
//!
//! The single tags are one fruit or vegetable per letter of the alphabet,
//! plus ten Japanese fruits. They are chosen so that every palette color is
//! the main color of six of them.
//!
//! Run: `cargo run --example gallery > gallery.html`

use picturesque_marks::{
    BackgroundStyle, Color, ColorName, ComposeLayout, SignatureStyle, UnderlineOptions,
    background_svg, icon_svg, signature_svg, underline_svg,
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
    &["パッションフルーツ", "card"],
    &["cardbox", "card"],
    &["cardbox", "card", "framework"],
    &["zucchini", "yuzu", "xigua", "watermelon", "vanilla"],
    &[],
];

// A few examples with a signature curve. Each entry is (signature, style, tags).
const SIGNED: &[(&str, SignatureStyle, &[&str])] = &[
    ("Alice", SignatureStyle::Crossing, &["banana"]),
    ("Alice", SignatureStyle::NoCrossing, &["banana"]),
    ("Bob", SignatureStyle::Crossing, &["banana"]),
    ("Bob", SignatureStyle::NoCrossing, &["banana"]),
    ("Alice", SignatureStyle::Face, &["banana"]),
    ("Bob", SignatureStyle::Face, &["banana"]),
    ("Alice", SignatureStyle::Crossing, &["cardbox", "card"]),
    (
        "Bob",
        SignatureStyle::NoCrossing,
        &["cardbox", "card", "framework"],
    ),
    (
        "Carol",
        SignatureStyle::Face,
        &["cardbox", "card", "framework"],
    ),
    ("Alice", SignatureStyle::Crossing, &[]),
];

fn figure(svg: &str, caption: &str) {
    println!(
        "<figure style=\"margin:4px;padding:8px;border:1px solid #ddd;border-radius:4px;\">{svg}<figcaption style=\"text-align:center;\">{caption}</figcaption></figure>"
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

    for style in [BackgroundStyle::Shapes, BackgroundStyle::Petal] {
        println!("<h2>Single tag ({style:?})</h2>");
        println!("<div style=\"display:flex;flex-wrap:wrap;\">");
        for tag in TAGS {
            figure(
                &background_svg(&[tag], ComposeLayout::default(), style, size, size),
                tag,
            );
        }
        println!("</div>");
    }

    for layout in [ComposeLayout::Grid, ComposeLayout::Overlay] {
        println!("<h2>Multiple tags ({layout:?})</h2>");
        println!("<div style=\"display:flex;flex-wrap:wrap;\">");
        for tags in TAG_SETS {
            figure(
                &background_svg(tags, layout, BackgroundStyle::default(), size, size),
                &tags_caption(tags),
            );
        }
        println!("</div>");
    }

    for background in [BackgroundStyle::Shapes, BackgroundStyle::Petal] {
        println!("<h2>With signature ({background:?})</h2>");
        println!("<div style=\"display:flex;flex-wrap:wrap;\">");
        for (signature, style, tags) in SIGNED {
            let svg = match tags {
                [tag] => icon_svg(
                    &[tag],
                    ComposeLayout::default(),
                    background,
                    signature,
                    *style,
                    Color::BLACK,
                    size,
                    size,
                ),
                _ => icon_svg(
                    tags,
                    ComposeLayout::default(),
                    background,
                    signature,
                    *style,
                    Color::BLACK,
                    size,
                    size,
                ),
            };
            figure(
                &svg,
                &format!("{} / sign: {signature} / {style:?}", tags_caption(tags)),
            );
        }
        println!("</div>");
    }

    println!("<h2>Signature</h2>");
    println!("<div style=\"display:flex;flex-wrap:wrap;\">");
    for (signature, style, color) in [
        ("Alice", SignatureStyle::Crossing, Color::BLACK),
        ("Alice", SignatureStyle::NoCrossing, Color::BLACK),
        ("Alice", SignatureStyle::Face, Color::BLACK),
        ("Alice", SignatureStyle::Face, ColorName::Blue.color()),
        ("Alice", SignatureStyle::Blob, ColorName::Red.color()),
        ("Bob", SignatureStyle::Blob, ColorName::Cyan.color()),
        ("Carol", SignatureStyle::Blob, Color::rgb(0x80, 0x80, 0x80)),
    ] {
        figure(
            &signature_svg(signature, style, color, size, size),
            &format!("sign: {signature} / {style:?} / {}", color.to_hex()),
        );
    }
    println!("</div>");

    // The underline takes its colors from the same tags as the icon. The seed stands for
    // the post slug; the heading lines of one post get the slug plus a number.
    println!("<h2>Underline</h2>");
    for tags in TAG_SETS {
        let icon = background_svg(
            tags,
            ComposeLayout::default(),
            BackgroundStyle::default(),
            size,
            size,
        );
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
