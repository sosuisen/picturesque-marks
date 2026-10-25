//! Generates an SVG from the command line and writes it to stdout.
//!
//! Usage:
//! - `picturesque-marks [--size W[xH]] [--signature TEXT] [--style crossing|no-crossing|face] [--layout grid|overlay] [TAG...]`
//! - `picturesque-marks underline [--spikes] [--class NAME] [--stroke-width W] SEED [TAG...]`

use std::process::ExitCode;

use picturesque_marks::{
    compose_icon, compose_ptag, underline_svg, ComposeLayout, ForegroundStyle, UnderlineOptions,
};

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1).peekable();
    if args.peek().is_some_and(|arg| arg == "underline") {
        args.next();
        return underline_main(args);
    }
    icon_main(args)
}

fn icon_main(mut args: impl Iterator<Item = String>) -> ExitCode {
    let mut tags: Vec<String> = Vec::new();
    let mut width = 100;
    let mut height = 100;
    let mut signature: Option<String> = None;
    let mut style = ForegroundStyle::default();
    let mut layout = ComposeLayout::default();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--help" | "-h" => {
                print_usage();
                return ExitCode::SUCCESS;
            }
            "--signature" | "-g" => {
                let Some(value) = args.next() else {
                    eprintln!("--signature requires a value");
                    print_usage();
                    return ExitCode::FAILURE;
                };
                signature = Some(value);
            }
            "--style" | "-t" => {
                let Some(value) = args.next() else {
                    eprintln!("--style requires a value");
                    print_usage();
                    return ExitCode::FAILURE;
                };
                style = match value.as_str() {
                    "crossing" => ForegroundStyle::Crossing,
                    "no-crossing" => ForegroundStyle::NoCrossing,
                    "face" => ForegroundStyle::Face,
                    _ => {
                        eprintln!("style must be crossing, no-crossing or face: {value}");
                        print_usage();
                        return ExitCode::FAILURE;
                    }
                };
            }
            "--layout" | "-l" => {
                let Some(value) = args.next() else {
                    eprintln!("--layout requires a value");
                    print_usage();
                    return ExitCode::FAILURE;
                };
                layout = match value.as_str() {
                    "grid" => ComposeLayout::Grid,
                    "overlay" => ComposeLayout::Overlay,
                    _ => {
                        eprintln!("layout must be grid or overlay: {value}");
                        print_usage();
                        return ExitCode::FAILURE;
                    }
                };
            }
            "--size" | "-s" => {
                let Some(value) = args.next() else {
                    eprintln!("--size requires a value");
                    print_usage();
                    return ExitCode::FAILURE;
                };
                match parse_size(&value) {
                    Some((w, h)) => {
                        width = w;
                        height = h;
                    }
                    None => {
                        eprintln!("size must be W or WxH with positive integers: {value}");
                        print_usage();
                        return ExitCode::FAILURE;
                    }
                }
            }
            _ => tags.push(arg),
        }
    }

    // No tags would produce the default icon, so use "A" like the original Program.Main.
    if tags.is_empty() {
        tags.push(String::from("A"));
    }

    let svg = match signature {
        Some(signature) => compose_icon(&tags, layout, &signature, style, width, height),
        None => compose_ptag(&tags, layout, width, height),
    };
    println!("{svg}");
    ExitCode::SUCCESS
}

fn underline_main(mut args: impl Iterator<Item = String>) -> ExitCode {
    let mut seed: Option<String> = None;
    let mut tags: Vec<String> = Vec::new();
    let mut options = UnderlineOptions::default();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--help" | "-h" => {
                print_underline_usage();
                return ExitCode::SUCCESS;
            }
            "--spikes" => options.spikes = true,
            "--class" | "-c" => {
                let Some(value) = args.next() else {
                    eprintln!("--class requires a value");
                    print_underline_usage();
                    return ExitCode::FAILURE;
                };
                options.class_name = value;
            }
            "--stroke-width" | "-w" => {
                let Some(value) = args.next() else {
                    eprintln!("--stroke-width requires a value");
                    print_underline_usage();
                    return ExitCode::FAILURE;
                };
                match value.parse::<f64>().ok().filter(|w| *w > 0.0) {
                    Some(w) => options.stroke_width = w,
                    None => {
                        eprintln!("stroke width must be a positive number: {value}");
                        print_underline_usage();
                        return ExitCode::FAILURE;
                    }
                }
            }
            _ if seed.is_none() => seed = Some(arg),
            _ => tags.push(arg),
        }
    }

    let Some(seed) = seed else {
        eprintln!("a seed is required");
        print_underline_usage();
        return ExitCode::FAILURE;
    };
    // The SVG text already ends with a newline.
    print!("{}", underline_svg(&seed, &tags, &options));
    ExitCode::SUCCESS
}

/// Parses `W` or `WxH`.
fn parse_size(value: &str) -> Option<(i32, i32)> {
    let positive = |s: &str| s.parse::<i32>().ok().filter(|n| *n > 0);
    match value.split_once('x') {
        Some((w, h)) => Some((positive(w)?, positive(h)?)),
        None => {
            let w = positive(value)?;
            Some((w, w))
        }
    }
}

fn print_usage() {
    eprintln!(
        "usage: picturesque-marks [--size W[xH]] [--signature TEXT] [--style STYLE] [--layout LAYOUT] [TAG...]"
    );
    eprintln!("       picturesque-marks underline [OPTIONS] SEED");
    eprintln!("  TAG                  tag string the background is generated from (default: A)");
    eprintln!(
        "                       up to four tags are composed into one icon, in the given order"
    );
    eprintln!("  --size, -s W[xH]     icon width and height (default: 100)");
    eprintln!("  --signature, -g TEXT draw a hand-drawn style curve generated from TEXT on top");
    eprintln!("  --style, -t STYLE    curve style: crossing (default), no-crossing or face");
    eprintln!("  --layout, -l LAYOUT  how several tags are composed (default: overlay)");
    eprintln!("                       grid: a 2x2 grid of tiles in the given order");
    eprintln!("                       overlay: the first tag at full size, the others as");
    eprintln!("                       half-transparent tiles over it");
    eprintln!("  underline            draw a sketchy underline instead; see `underline --help`");
}

fn print_underline_usage() {
    eprintln!(
        "usage: picturesque-marks underline [--spikes] [--class NAME] [--stroke-width W] SEED [TAG...]"
    );
    eprintln!("  SEED                 string the shape is generated from");
    eprintln!("  TAG                  tags the stroke colors come from, as in the tag icon");
    eprintln!("                       without tags, the colors come from the palette");
    eprintln!("  --spikes             shake the line near its right end");
    eprintln!("  --class, -c NAME     class attribute on the root element");
    eprintln!("  --stroke-width, -w W stroke width in screen pixels (default: 3)");
}
