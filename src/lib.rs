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
            decoration: _,
            accidental: _,
            note: _,
            octave: _,
            length,
        } => length.clone(),
        MusicSymbol::Bar(_) => 1.0,
        MusicSymbol::VisualBreak() => 1.0,
        _ => 0.0, // mysterious
    }
}

// How much space does this symbol get?
fn get_space(sym: &MusicSymbol, total_weight: f32, available_px: i32) -> f32 {
    match sym {
        MusicSymbol::Note {
            decoration,
            accidental,
            note,
            octave,
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

// Two coord systems vs one?
// Currently, one.
// Do logic in terms of where it sits in the line, and then adjust for margin and other factors.
fn render_sym(
    sym: &MusicSymbol,
    pos: f32,
    config: &LayoutConfig,
    available_px: i32,
    total_weight: f32,
) -> Vec<Box<dyn Node>> {
    let mut nodes: Vec<Box<dyn Node>> = Vec::new();

    let (x, y) = pos_to_coord(pos, config, available_px, total_weight);
    match sym {
        MusicSymbol::Note {
            decoration,
            accidental,
            note,
            octave,
            length,
        } => {
            let note_offset = match note {
                'C' => -2,
                'D' => -1,
                'E' => 0,
                'F' => 1,
                'G' => 2,
                'A' => 3,
                'B' => 4,
                'c' => 5,
                'd' => 5,
                'e' => 6,
                'f' => 7,
                'g' => 8,
                'a' => 9,
                'b' => 10,
                _ => panic!("Unexpected char '{}'", note),
            };
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

    //nodes.push(text_node_create('\u{E0A4}', 20.0, 20.0));

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

    #[test]
    fn it_works() {
        assert_eq!(placeholder(), "abcrend - coming soon!");
    }
}
