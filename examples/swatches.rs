//! Generates a swatch sheet: every palette color crossed with every hatch pattern.
//! Use it to tune the colors and the patterns by eye.
//!
//! Run: `cargo run --example swatches > swatches.html`

use picturesque_marks::{ColorName, HatchPatternType, IconBuilder, ShapeType};

/// Edge length of one swatch in pixels. The same size as a gallery icon,
/// so the pattern density looks as it does in real icons.
const SIZE: i32 = 128;

/// One swatch: a full-size shape filled with the hatch in the color.
///
/// Every icon names its pattern `pat0`. Inline SVGs share one HTML document,
/// so the ids would collide and every swatch would show the first pattern.
/// Rename the id per swatch to keep them apart.
fn swatch(shape: ShapeType, color: ColorName, hatch: HatchPatternType) -> String {
    let half = (SIZE / 2) as f32;
    let mut builder = IconBuilder::new(SIZE, SIZE);
    match shape {
        ShapeType::Circle => builder.add_circle(hatch, color.color(), half, 0.0, 0.0),
        ShapeType::Diamond => builder.add_diamond(hatch, color.color(), half, 0.0, 0.0),
        ShapeType::Pentagon => builder.add_pentagon(hatch, color.color(), half, 0.0, 0.0),
    };
    let unique = format!("pat_{shape:?}_{color:?}_{hatch:?}");
    builder.to_svg().replace("pat0", &unique)
}

/// One matrix: colors down, hatch patterns across, all in the given shape.
fn matrix(shape: ShapeType) {
    println!("<h2>{shape:?}</h2>");
    println!("<table>");
    println!("<tr><th></th>");
    for hatch in HatchPatternType::ALL {
        println!("<th class=\"hatch\">{hatch:?}</th>");
    }
    println!("</tr>");

    for color in ColorName::ALL {
        let hex = color.color().to_hex();
        println!("<tr>");
        println!("<th class=\"color\"><span class=\"chip\" style=\"background:{hex}\"></span>{color:?} {hex}</th>");
        for hatch in HatchPatternType::ALL {
            println!("<td>{}</td>", swatch(shape, color, hatch));
        }
        println!("</tr>");
    }
    println!("</table>");
}

fn main() {
    println!("<!DOCTYPE html>");
    println!("<html><head><meta charset=\"utf-8\"><title>Picturesque Marks Swatches</title>");
    println!("<style>");
    println!("body{{font-family:sans-serif;margin:16px;}}");
    println!("table{{border-collapse:collapse;}}");
    println!("th,td{{padding:4px;text-align:center;font-size:12px;font-weight:normal;}}");
    println!("th.color{{text-align:left;white-space:nowrap;}}");
    println!("th.hatch{{writing-mode:vertical-rl;transform:rotate(180deg);}}");
    println!(".chip{{display:inline-block;width:12px;height:12px;vertical-align:middle;margin-right:4px;}}");
    println!("</style></head><body>");
    println!("<h1>Picturesque Marks Swatches</h1>");
    println!("<p>One matrix per shape. Rows: palette colors. Columns: hatch patterns. Each swatch is {SIZE}px.</p>");

    for shape in ShapeType::ALL {
        matrix(shape);
    }
    println!("</body></html>");
}
