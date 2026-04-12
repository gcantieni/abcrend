//! ABC music notation to SVG renderer
//!
//! This crate will convert ABC notation to SVG musical scores.

//use std::alloc::Layout;

use abc_parser::abc;
//use abc_parser::abc; // TODO: Why doesn't this import anything?
use abc_parser::datatypes::*;
use svg::Document;
use svg::Node;
use svg::node::element::{Circle, Line, Polygon, Text};

// TODO: remove
// but I don't care about testing rn

/// Placeholder function
pub fn placeholder() -> &'static str {
    "abcrend - coming soon!"
}

// This should be scalable, so lay everything out in terms of
// the height of one base unit. Font size must be adjusted such that BASE_UNIT pixels correspond to
// not head height. That way adding one unit to the note y position will make it one higher in the scale.
//
const BASE_UNIT: f32 = 8.0;

pub struct LayoutConfig {
    pub file_name: String,
    pub margin_left: f32,
    pub margin_top: f32,
}

enum StemType {
    Up,
    Down,
}

// Ignore everything but length (for now); just return that.
fn get_sym_weight(sym: &MusicSymbol) -> f32 {
    match sym {
        MusicSymbol::Note {
            tie: _,
            accidental: _,
            decorations: _,
            note: _,
            octave: _,
            length,
        } => length.clone(),
        MusicSymbol::Bar(_) => 1.0,
        MusicSymbol::VisualBreak => 1.0,
        _ => 0.0, // mysterious
    }
}

// How much space does this symbol get?
fn get_space(sym: &MusicSymbol, total_weight: f32, available_px: i32) -> f32 {
    match sym {
        MusicSymbol::Note {
            decorations: _,
            tie: _,
            accidental: _,
            note: _,
            octave: _,
            length,
        } => length.clone(),
        _ => 1.0,
    }
}

// Takes in the logical position, outputs the actual x coordinate that a symbol
// should be drawn at.
fn pos_to_coord(
    pos: f32,
    config: &LayoutConfig,
    available_px: i32,
    total_weight: f32,
) -> (f32, f32) {
    (
        config.margin_left + pos * BASE_UNIT * available_px as f32 / total_weight,
        config.margin_top,
    )
}

fn logical_y_to_coord_y(logical_y: f32, config: &LayoutConfig) -> f32 {
    config.margin_top + (4.0 - logical_y) * BASE_UNIT
}

// TODO: Rework: logical X shouldn't account for available space or weight.
// that's someone else's job.
fn logical_x_to_coord_x(
    logical_x: f32,
    config: &LayoutConfig,
    available_px: i32,
    total_weght: f32,
) -> f32 {
    config.margin_left + logical_x * BASE_UNIT * available_px as f32 / total_weght
}

// Each note increment is 0.5 of a quarter note e.g.
// 0.5 of a quarter note.
fn get_note_offset(note: &Note, octave: i8) -> f32 {
    let base = match note {
        Note::C => -2.0,
        Note::D => -1.0,
        Note::E => 0.0,
        Note::F => 1.0,
        Note::G => 2.0,
        Note::A => 3.0,
        Note::B => 4.0,
    };
    (base + (octave - 1) as f32 * 7.0) * 0.5
}

fn render_sym(
    sym: &MusicSymbol,
    logical_x: f32,
    config: &LayoutConfig,
    available_px: i32,
    total_weight: f32,
) -> Vec<Box<dyn Node>> {
    let mut nodes: Vec<Box<dyn Node>> = Vec::new();

    match sym {
        MusicSymbol::Note {
            decorations: _,
            tie: _,
            accidental: _,
            note,
            octave,
            length,
        } => {
            let x = logical_x_to_coord_x(logical_x, config, available_px, total_weight);
            let note_offset = get_note_offset(note, *octave);
            let y = logical_y_to_coord_y(note_offset, config);
            // TODO: if note offset is less than zero and match checks out,
            //       add a line throught it.

            // Draw note head if it's a short enough note to have a head.
            if *length > 0.0 && *length < 2.0 {
                nodes.push(text_node_create('\u{E0A4}', x, y));
            } else if *length == 2.0 {
                nodes.push(text_node_create('\u{E0A3}', x, y));
            } else if *length == 4.0 {
                nodes.push(text_node_create('\u{E0A2}', x, y));
            }
        }
        _ => {}
    }

    return nodes;
}

pub fn render_abc(abc_str: &str, config: LayoutConfig) -> svg::Document {
    let mut tune_book = match abc::tune_book(abc_str) {
        Ok(tb) => tb,
        Err(error) => panic!("Problem parsing tune book: {error}"),
    };

    let mut tune = tune_book.tunes.remove(0);
    let body = tune.body.take().expect("No tune body");
    let _header = tune.header;

    // Determine this.. somehow.
    let available_width = 30.0;

    let mut nodes: Vec<Box<dyn Node>> = Vec::new();

    // Currently, lines don't affect one another. Render them one at a time.
    // Get total weight, divide it up proportional to note duration. Do math such that
    // note plus adjusted distance = total weight.

    // In the second pass, render each symbol.
    for line in body.music {
        let total_weight: f32 = line
            .symbols
            .iter()
            .fold(0.0, |weight, sym| weight + get_sym_weight(sym));
        println!("Total weight for line is {}", total_weight);

        // The gclef records its position based on the mid-point of its back.
        // Thus it can be aligned with the lines of the staff and it looks about right.
        let gclef = text_node_create(
            '\u{E050}',
            config.margin_left,
            config.margin_top + 3.0 * BASE_UNIT,
        );
        nodes.push(gclef);

        // The actual lines, seaparated by the width of a note
        let line_stroke_width = 0.1 * BASE_UNIT;
        let line_length = BASE_UNIT * available_width;
        for i in 0..5 {
            nodes.push(Box::new(
                Line::new()
                    .set("x1", config.margin_left)
                    .set("y1", config.margin_top + (i as f32 * BASE_UNIT))
                    .set("x2", config.margin_left + line_length)
                    .set("y2", config.margin_top + (i as f32 * BASE_UNIT))
                    .set("stroke", "black")
                    .set("stroke-width", line_stroke_width),
            ));
        }

        // Offset the first note because of the cleff.
        let mut current_pos = 1.0;
        for sym in line.symbols {
            push_svg_vec(
                &mut nodes,
                render_sym(
                    &sym,
                    current_pos,
                    &config,
                    available_width as i32,
                    total_weight,
                ),
            );
            current_pos += get_space(&sym, total_weight, available_width as i32);
        }
    }

    let mut doc = Document::new()
        .set("viewBox", (0, 0, 300, 300))
        .set("font-family", "Bravura");

    for n in nodes {
        doc = doc.add(n);
    }

    svg::save("example.svg", &doc).unwrap();

    return doc;
}

// Can be useful to see where exactly a point is when we are working with fonts
fn _debug_draw_dot(cx: f32, cy: f32, r: f32) -> Box<dyn Node> {
    return Box::new(
        Circle::new()
            .set("cx", cx)
            .set("cy", cy)
            .set("fill", "red")
            .set("r", r * BASE_UNIT),
    );
}

fn text_node_create(c: char, x: f32, y: f32) -> Box<dyn Node> {
    return Box::new(
        Text::new(c)
            .set("x", x)
            .set("y", y)
            .set("font-size", 4.0 * BASE_UNIT),
    );
}

fn bar_create(_x1: f32, _y1: f32, _x2: f32, _y2: f32) -> Box<dyn Node> {
    return Box::new(
        Polygon::new()
            .set("points", "60,60 70,60 70,80 60,80")
            .set("fill", "red"),
    );
}

// This required some minute tweaking to make the stem overlap to the right degree.
fn stem_draw(note_x: f32, note_y: f32, t: StemType, length: f32) -> Vec<Box<dyn Node>> {
    let mut nodes = Vec::new();
    let x: f32;
    let y: f32;
    let flag: Option<char>;

    match t {
        StemType::Up => {
            x = note_x + (BASE_UNIT * 1.11); // Approx note + 1
            y = note_y - 0.1 * BASE_UNIT;
            flag = match length {
                0.5 => Some('\u{E240}'),
                0.25 => Some('\u{E242}'),
                0.125 => Some('\u{E244}'),
                0.0625 => Some('\u{E246}'),
                0.03125 => Some('\u{E248}'),
                0.015625 => Some('\u{E24A}'),
                _ => None,
            }
        }
        StemType::Down => {
            x = note_x + 0.06 * BASE_UNIT;
            y = note_y + 3.60 * BASE_UNIT; // A stem is approx 3 notes high
            flag = match length {
                0.5 => Some('\u{E241}'),
                0.25 => Some('\u{E243}'),
                0.125 => Some('\u{E245}'),
                0.0625 => Some('\u{E247}'),
                0.03125 => Some('\u{E249}'),
                0.015625 => Some('\u{E24B}'),
                _ => None,
            }
        }
    };

    // skip stem for whole note
    if length != 4.0 {
        nodes.push(text_node_create('\u{E210}', x, y));
    }

    if let Some(f) = flag {
        nodes.push(text_node_create(f, x, y));
    }

    return nodes;
}

// Simple helper to avoid a tmp variable.
fn push_svg_vec(vec1: &mut Vec<Box<dyn Node>>, vec2: Vec<Box<dyn Node>>) {
    for v in vec2 {
        vec1.push(v);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::{BASE_UNIT, get_note_offset, logical_x_to_coord_x, logical_y_to_coord_y};

    fn cfg() -> LayoutConfig {
        LayoutConfig {
            file_name: String::from("test.svg"),
            margin_left: 10.0,
            margin_top: 20.0,
        }
    }

    // All 14 notes in ascending pitch order (note, octave).
    const ASCENDING: [(Note, i8); 14] = [
        (Note::C, 1),
        (Note::D, 1),
        (Note::E, 1),
        (Note::F, 1),
        (Note::G, 1),
        (Note::A, 1),
        (Note::B, 1),
        (Note::C, 2),
        (Note::D, 2),
        (Note::E, 2),
        (Note::F, 2),
        (Note::G, 2),
        (Note::A, 2),
        (Note::B, 2),
    ];

    // --- get_note_offset ---

    // The offset of each note must be strictly greater than the one before it.
    #[test]
    fn note_offsets_increase_with_pitch() {
        let offsets: Vec<f32> = ASCENDING
            .iter()
            .map(|&(n, o)| get_note_offset(&n, o))
            .collect();
        for w in offsets.windows(2) {
            assert!(w[0] < w[1], "expected {} < {}", w[0], w[1]);
        }
    }

    // Every adjacent step in the diatonic sequence is exactly 0.5 units.
    #[test]
    fn note_offsets_are_uniform_half_unit_steps() {
        let offsets: Vec<f32> = ASCENDING
            .iter()
            .map(|&(n, o)| get_note_offset(&n, o))
            .collect();
        for w in offsets.windows(2) {
            assert!((w[1] - w[0] - 0.5).abs() < 1e-6, "step was {}", w[1] - w[0]);
        }
    }

    // E is the reference pitch; its offset is 0.
    #[test]
    fn e4_offset_is_zero() {
        assert_eq!(get_note_offset(&Note::E, 1), 0.0);
    }

    // --- logical_y_to_coord_y ---

    // Higher pitch → smaller SVG y (SVG y increases downward).
    #[test]
    fn higher_pitch_has_smaller_svg_y() {
        let cfg = cfg();
        let e_y = logical_y_to_coord_y(get_note_offset(&Note::E, 1), &cfg);
        let g_y = logical_y_to_coord_y(get_note_offset(&Note::G, 1), &cfg);
        assert!(g_y < e_y, "G should render above E on the staff");
    }

    // Each 1.0 increment in logical_y moves exactly BASE_UNIT pixels upward.
    #[test]
    fn y_step_equals_base_unit() {
        let cfg = cfg();
        let y0 = logical_y_to_coord_y(0.0, &cfg);
        let y1 = logical_y_to_coord_y(1.0, &cfg);
        assert!((y0 - y1 - BASE_UNIT).abs() < 1e-6);
    }

    // Scaling margin_top shifts every y coordinate by the same amount.
    #[test]
    fn y_coord_shifts_with_margin_top() {
        let cfg1 = cfg();
        let cfg2 = LayoutConfig {
            margin_top: cfg1.margin_top + 15.0,
            ..cfg()
        };
        for &(n, o) in &ASCENDING {
            let off = get_note_offset(&n, o);
            let delta = logical_y_to_coord_y(off, &cfg1) - logical_y_to_coord_y(off, &cfg2);
            assert!((delta + 15.0).abs() < 1e-6);
        }
    }

    // --- logical_x_to_coord_x ---

    // At logical_x = 0 the result is exactly margin_left.
    #[test]
    fn x_at_origin_equals_margin_left() {
        let cfg = cfg();
        let x = logical_x_to_coord_x(0.0, &cfg, 100, 4.0);
        assert!((x - cfg.margin_left).abs() < 1e-6);
    }

    // A later position always maps to a larger x coordinate.
    #[test]
    fn later_position_is_further_right() {
        let cfg = cfg();
        let x1 = logical_x_to_coord_x(1.0, &cfg, 100, 4.0);
        let x2 = logical_x_to_coord_x(2.0, &cfg, 100, 4.0);
        assert!(x2 > x1);
    }

    // The mapping is linear: equal logical steps produce equal pixel steps.
    #[test]
    fn x_mapping_is_linear() {
        let cfg = cfg();
        let x0 = logical_x_to_coord_x(0.0, &cfg, 100, 4.0);
        let x1 = logical_x_to_coord_x(1.0, &cfg, 100, 4.0);
        let x2 = logical_x_to_coord_x(2.0, &cfg, 100, 4.0);
        assert!((x2 - x1 - (x1 - x0)).abs() < 1e-6);
    }

    // Scaling margin_left shifts every x coordinate by the same amount.
    #[test]
    fn x_coord_shifts_with_margin_left() {
        let cfg1 = cfg();
        let cfg2 = LayoutConfig {
            margin_left: cfg1.margin_left + 15.0,
            ..cfg()
        };
        for pos in [0.0_f32, 1.0, 2.5] {
            let delta = logical_x_to_coord_x(pos, &cfg2, 100, 4.0)
                - logical_x_to_coord_x(pos, &cfg1, 100, 4.0);
            assert!((delta - 15.0).abs() < 1e-6);
        }
    }

    // --- upstream parser compatibility ---

    #[test]
    fn cooleys_parses() {
        let input =
            "X: 1\nT: Cooley's\nR: reel\nM: 4/4\nL: 1/8\nK: Edor\n|:D2|EBBA B2 EB|B2 AB dBAG:|\n";
        assert!(abc::tune_book(input).is_ok());
    }
}
