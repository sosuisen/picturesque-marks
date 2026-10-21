//! The expected SVGs were produced by the original `title-underline.mjs` of
//! sosuisen.net, which took colors instead of tags. The Rust port must give
//! the same shape, byte for byte, so that a re-sync of the site does not
//! redraw every line. The script was given placeholder colors, and the test
//! swaps them for the colors the crate picks from the tags.

use picturesque_marks::{UnderlineOptions, underline_colors, underline_svg};

const PLACEHOLDERS: [&str; 3] = ["#C0L0R0", "#C0L0R1", "#C0L0R2"];

fn check(seed: &str, tags: &[&str], class_name: &str, stroke_width: f64, spikes: bool, expected: &str) {
    let options = UnderlineOptions {
        class_name: class_name.to_string(),
        stroke_width,
        spikes,
    };
    let mut expected = expected.to_string();
    if let Some(colors) = underline_colors(tags) {
        for (placeholder, color) in PLACEHOLDERS.iter().zip(colors) {
            expected = expected.replace(placeholder, &color.to_hex());
        }
    }
    assert_eq!(underline_svg(seed, tags, &options), expected, "seed {seed:?}");
}

#[test]
fn case_0_2026_10_16() {
    check(
        "2026-10-16",
        &[],
        "",
        3.0,
        true,
        "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 1000 40\" preserveAspectRatio=\"none\"><path d=\"M5.5,16.2 C273.6,20.1 541.8,21.9 809.9,22.5 L819.3,31.1 L828.7,22.6 L837.6,14.3 L846.6,22.8 L851.5,35.8 L856.4,22.8 L977,23.8\" stroke=\"#EB9BEB\" stroke-width=\"3\" stroke-opacity=\"0.8\" stroke-linecap=\"round\" fill=\"none\" vector-effect=\"non-scaling-stroke\"/><path d=\"M3.7,15.5 C287.8,10.1 571.9,17.2 856,23.3 L861.3,7.3 L866.6,23.4 L872.4,35.7 L878.2,23.5 L882.3,9.2 L886.4,23.5 L894.2,33.1 L902,23.7 L992.5,24.5\" stroke=\"#EB9B9B\" stroke-width=\"3\" stroke-opacity=\"0.8\" stroke-linecap=\"round\" fill=\"none\" vector-effect=\"non-scaling-stroke\"/><path d=\"M4.3,18.6 C272.9,26.7 541.5,27.1 810.1,20.9 L819.2,8 L828.2,21 L833.6,30.8 L839,21 L848.1,13.7 L857.2,21 L861.4,36.2 L865.6,21.1 L875.3,7.7 L885.1,21.1 L992.2,21.4\" stroke=\"#EBD89B\" stroke-width=\"3\" stroke-opacity=\"0.8\" stroke-linecap=\"round\" fill=\"none\" vector-effect=\"non-scaling-stroke\"/></svg>\n",
    );
}

#[test]
fn case_1_2026_10_16_h2_0() {
    check(
        "2026-10-16#h2-0",
        &["Computer", "CG"],
        "heading-underline",
        1.5,
        false,
        "<svg xmlns=\"http://www.w3.org/2000/svg\" class=\"heading-underline\" viewBox=\"0 0 1000 40\" preserveAspectRatio=\"none\"><path d=\"M15.7,23.1 C334.6,28.6 653.6,26.8 972.5,16.9\" stroke=\"#C0L0R0\" stroke-width=\"1.5\" stroke-opacity=\"0.8\" stroke-linecap=\"round\" fill=\"none\" vector-effect=\"non-scaling-stroke\"/><path d=\"M24.2,18.9 C339.8,21.2 655.5,33.1 971.1,21.1\" stroke=\"#C0L0R1\" stroke-width=\"1.5\" stroke-opacity=\"0.8\" stroke-linecap=\"round\" fill=\"none\" vector-effect=\"non-scaling-stroke\"/><path d=\"M11.6,22.5 C340.1,27.1 668.6,8.3 997,17.5\" stroke=\"#C0L0R2\" stroke-width=\"1.5\" stroke-opacity=\"0.8\" stroke-linecap=\"round\" fill=\"none\" vector-effect=\"non-scaling-stroke\"/></svg>\n",
    );
}

#[test]
fn case_2_coding_gear_ratio() {
    check(
        "coding-gear-ratio",
        &["development", "agent"],
        "",
        3.0,
        true,
        "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 1000 40\" preserveAspectRatio=\"none\"><path d=\"M16.9,18 C286.7,27.5 556.5,20.2 826.3,21.3 L830.7,28.5 L835,21.4 L842,3.9 L848.9,21.4 L858.1,28.7 L867.2,21.5 L874.9,2.6 L882.7,21.6 L994.1,22\" stroke=\"#C0L0R0\" stroke-width=\"3\" stroke-opacity=\"0.8\" stroke-linecap=\"round\" fill=\"none\" vector-effect=\"non-scaling-stroke\"/><path d=\"M12.7,17.1 C288.3,11.3 563.9,16 839.5,22 L847.9,33.3 L856.2,22.1 L861.7,9.1 L867.1,22.2 L871.2,38.3 L875.3,22.2 L990.9,22.9\" stroke=\"#C0L0R1\" stroke-width=\"3\" stroke-opacity=\"0.8\" stroke-linecap=\"round\" fill=\"none\" vector-effect=\"non-scaling-stroke\"/><path d=\"M3.5,20.8 C281.7,31 560,22.9 838.3,19.4 L844.2,11.3 L850,19.4 L854.8,35.8 L859.7,19.4 L868.3,8.6 L876.9,19.4 L980.6,19.2\" stroke=\"#C0L0R2\" stroke-width=\"3\" stroke-opacity=\"0.8\" stroke-linecap=\"round\" fill=\"none\" vector-effect=\"non-scaling-stroke\"/></svg>\n",
    );
}

#[test]
fn case_3_coding_gear_ratio_h2_3() {
    check(
        "coding-gear-ratio#h2-3",
        &["identicon"],
        "heading-underline",
        1.5,
        true,
        "<svg xmlns=\"http://www.w3.org/2000/svg\" class=\"heading-underline\" viewBox=\"0 0 1000 40\" preserveAspectRatio=\"none\"><path d=\"M9.5,16 C271.1,17.2 532.7,26.2 794.3,22.4 L799.7,10.3 L805.1,22.5 L812.5,34.8 L819.9,22.6 L829.5,11 L839.2,22.7 L848.7,30.6 L858.2,22.9 L998.1,24\" stroke=\"#C0L0R0\" stroke-width=\"1.5\" stroke-opacity=\"0.8\" stroke-linecap=\"round\" fill=\"none\" vector-effect=\"non-scaling-stroke\"/><path d=\"M22.7,20.9 C286.8,29.4 550.9,31.4 815.1,19.4 L824.7,38.4 L834.4,19.4 L844.2,5.8 L854,19.3 L860.6,32.3 L867.2,19.3 L871.7,2 L876.2,19.3 L987.3,19.1\" stroke=\"#C0L0R1\" stroke-width=\"1.5\" stroke-opacity=\"0.8\" stroke-linecap=\"round\" fill=\"none\" vector-effect=\"non-scaling-stroke\"/><path d=\"M28.9,19.1 C294.3,21.8 559.7,28.6 825.1,20.6 L830.8,38.1 L836.4,20.6 L842.2,3.1 L847.9,20.6 L853.3,34.9 L858.8,20.6 L863.4,4.3 L868,20.6 L873,35.6 L878,20.7 L986,20.9\" stroke=\"#C0L0R2\" stroke-width=\"1.5\" stroke-opacity=\"0.8\" stroke-linecap=\"round\" fill=\"none\" vector-effect=\"non-scaling-stroke\"/></svg>\n",
    );
}

#[test]
fn case_4_________() {
    check(
        "日本語のスラッグ",
        &["フリーライティング", "cardbox", "card", "framework"],
        "",
        3.0,
        true,
        "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 1000 40\" preserveAspectRatio=\"none\"><path d=\"M18.2,18.3 C294.6,29.2 571,11.6 847.4,21.2 L855,28.8 L862.5,21.3 L868.3,12.4 L874,21.3 L881.2,32.8 L888.4,21.4 L894.7,3.1 L901.1,21.4 L985.6,21.7\" stroke=\"#C0L0R0\" stroke-width=\"3\" stroke-opacity=\"0.8\" stroke-linecap=\"round\" fill=\"none\" vector-effect=\"non-scaling-stroke\"/><path d=\"M25.5,16.3 C279.1,12 532.6,31.4 786.2,22.2 L795.9,10.7 L805.6,22.4 L813,39 L820.5,22.5 L829.8,9.4 L839.1,22.6 L978.6,23.7\" stroke=\"#C0L0R1\" stroke-width=\"3\" stroke-opacity=\"0.8\" stroke-linecap=\"round\" fill=\"none\" vector-effect=\"non-scaling-stroke\"/><path d=\"M16.7,23.5 C295.1,15 573.5,18.1 852,17.4 L861.5,4.8 L871,17.2 L880.8,24.2 L890.6,17.1 L898.4,3.6 L906.1,17 L975.3,16.5\" stroke=\"#C0L0R2\" stroke-width=\"3\" stroke-opacity=\"0.8\" stroke-linecap=\"round\" fill=\"none\" vector-effect=\"non-scaling-stroke\"/></svg>\n",
    );
}

#[test]
fn case_5_a() {
    check(
        "a",
        &[],
        "",
        3.0,
        false,
        "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 1000 40\" preserveAspectRatio=\"none\"><path d=\"M22.5,15.8 C341.2,29.6 659.8,22.8 978.4,24.2\" stroke=\"#9B9BEB\" stroke-width=\"3\" stroke-opacity=\"0.8\" stroke-linecap=\"round\" fill=\"none\" vector-effect=\"non-scaling-stroke\"/><path d=\"M27.3,23.9 C344.7,9.7 662,17.9 979.4,16.1\" stroke=\"#9B9BEB\" stroke-width=\"3\" stroke-opacity=\"0.8\" stroke-linecap=\"round\" fill=\"none\" vector-effect=\"non-scaling-stroke\"/><path d=\"M20.9,16.4 C343.7,25.4 666.4,11.1 989.2,23.6\" stroke=\"#9B9BEB\" stroke-width=\"3\" stroke-opacity=\"0.8\" stroke-linecap=\"round\" fill=\"none\" vector-effect=\"non-scaling-stroke\"/></svg>\n",
    );
}

#[test]
fn case_6_b() {
    check(
        "b",
        &[],
        "",
        3.0,
        true,
        "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 1000 40\" preserveAspectRatio=\"none\"><path d=\"M25.2,16.4 C277.2,24 529.1,30.2 781,22 L789.9,11.7 L798.7,22.2 L803.4,39 L808,22.2 L812.7,13.6 L817.4,22.3 L824.5,37.8 L831.5,22.4 L836.6,9.6 L841.7,22.5 L996.7,23.6\" stroke=\"#9BEBEB\" stroke-width=\"3\" stroke-opacity=\"0.8\" stroke-linecap=\"round\" fill=\"none\" vector-effect=\"non-scaling-stroke\"/><path d=\"M2.6,22.8 C266.1,26.3 529.5,26.3 793,18.3 L798,27.3 L803,18.2 L812.4,7.9 L821.9,18.1 L826.9,25.8 L831.9,18.1 L978,17.2\" stroke=\"#EB9BEB\" stroke-width=\"3\" stroke-opacity=\"0.8\" stroke-linecap=\"round\" fill=\"none\" vector-effect=\"non-scaling-stroke\"/><path d=\"M5,23.7 C271.3,18.9 537.7,27.8 804.1,17.6 L808.4,35 L812.8,17.6 L821.8,2.9 L830.8,17.4 L836.1,32.5 L841.4,17.4 L850.2,7.6 L859.1,17.2 L975,16.3\" stroke=\"#EBD89B\" stroke-width=\"3\" stroke-opacity=\"0.8\" stroke-linecap=\"round\" fill=\"none\" vector-effect=\"non-scaling-stroke\"/></svg>\n",
    );
}

#[test]
fn case_7_c() {
    check(
        "c",
        &["xyz"],
        "",
        3.0,
        true,
        "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 1000 40\" preserveAspectRatio=\"none\"><path d=\"M14,20.7 C290.5,31.2 567,29.1 843.4,19.5 L850,30 L856.6,19.5 L862.7,5.3 L868.7,19.4 L877.1,34.1 L885.5,19.4 L894.4,10.7 L903.2,19.4 L912.8,32.4 L922.5,19.4 L989.9,19.3\" stroke=\"#C0L0R0\" stroke-width=\"3\" stroke-opacity=\"0.8\" stroke-linecap=\"round\" fill=\"none\" vector-effect=\"non-scaling-stroke\"/><path d=\"M4.2,24.6 C273.8,21.7 543.5,19 813.1,17.1 L817.8,8.5 L822.4,17 L828.5,24.6 L834.7,16.9 L839.8,1.2 L844.9,16.8 L854.5,24.5 L864,16.6 L995.6,15.4\" stroke=\"#C0L0R1\" stroke-width=\"3\" stroke-opacity=\"0.8\" stroke-linecap=\"round\" fill=\"none\" vector-effect=\"non-scaling-stroke\"/><path d=\"M25.6,20.1 C301.9,16.6 578.3,15.2 854.6,19.9 L860.1,2.5 L865.5,19.9 L870.1,34.8 L874.6,19.9 L884.2,1.2 L893.7,19.9 L900.2,32.1 L906.7,19.9 L976.4,19.9\" stroke=\"#C0L0R2\" stroke-width=\"3\" stroke-opacity=\"0.8\" stroke-linecap=\"round\" fill=\"none\" vector-effect=\"non-scaling-stroke\"/></svg>\n",
    );
}

#[test]
fn case_8_d() {
    check(
        "d",
        &["abcdefghij"],
        "",
        3.0,
        true,
        "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 1000 40\" preserveAspectRatio=\"none\"><path d=\"M22.6,19 C291.2,25.7 559.9,16.6 828.5,20.6 L834,39 L839.6,20.7 L845.4,10.3 L851.2,20.7 L856.1,30.6 L861,20.7 L865.6,3.2 L870.2,20.7 L988.7,21\" stroke=\"#C0L0R0\" stroke-width=\"3\" stroke-opacity=\"0.8\" stroke-linecap=\"round\" fill=\"none\" vector-effect=\"non-scaling-stroke\"/><path d=\"M3,22.5 C285.6,10.7 568.2,10.7 850.8,18.1 L855.5,10 L860.2,18.1 L869.9,31.1 L879.6,18 L884.1,10.7 L888.6,17.9 L972.9,17.5\" stroke=\"#C0L0R1\" stroke-width=\"3\" stroke-opacity=\"0.8\" stroke-linecap=\"round\" fill=\"none\" vector-effect=\"non-scaling-stroke\"/><path d=\"M29.6,16.2 C285.1,13.5 540.6,24.4 796.1,22.4 L804.9,29.8 L813.6,22.5 L820.5,9.8 L827.3,22.6 L834.1,29.8 L840.9,22.7 L850.3,12.2 L859.8,22.9 L974.2,23.8\" stroke=\"#C0L0R2\" stroke-width=\"3\" stroke-opacity=\"0.8\" stroke-linecap=\"round\" fill=\"none\" vector-effect=\"non-scaling-stroke\"/></svg>\n",
    );
}

#[test]
fn case_9_e() {
    check(
        "e",
        &["zucchini", "yuzu", "xigua", "watermelon", "vanilla"],
        "",
        3.0,
        true,
        "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 1000 40\" preserveAspectRatio=\"none\"><path d=\"M29,17 C301,21.2 573,9.8 845,22.1 L851.5,39 L857.9,22.2 L865.1,6 L872.2,22.3 L879.7,34.8 L887.1,22.4 L891.3,4.2 L895.5,22.5 L974,23\" stroke=\"#C0L0R0\" stroke-width=\"3\" stroke-opacity=\"0.8\" stroke-linecap=\"round\" fill=\"none\" vector-effect=\"non-scaling-stroke\"/><path d=\"M28.7,22.3 C289.7,16.5 550.8,18.2 811.8,18.5 L820.6,36.1 L829.4,18.4 L837.3,8 L845.2,18.4 L851.9,36.6 L858.7,18.3 L863.1,9.1 L867.5,18.3 L876.1,36.3 L884.8,18.2 L980.7,17.7\" stroke=\"#C0L0R1\" stroke-width=\"3\" stroke-opacity=\"0.8\" stroke-linecap=\"round\" fill=\"none\" vector-effect=\"non-scaling-stroke\"/><path d=\"M27.6,22.7 C285.5,26.1 543.5,13.5 801.4,18.3 L811.1,7.5 L820.9,18.1 L826.3,26.3 L831.6,18.1 L839.4,3.7 L847.2,18 L972.6,17.3\" stroke=\"#C0L0R2\" stroke-width=\"3\" stroke-opacity=\"0.8\" stroke-linecap=\"round\" fill=\"none\" vector-effect=\"non-scaling-stroke\"/></svg>\n",
    );
}
