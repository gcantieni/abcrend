//! ABC music notation to SVG renderer
//!
//! This crate will convert ABC notation to SVG musical scores.

//use std::alloc::Layout;

use abc_parser::abc;
//use abc_parser::abc; // TODO: Why doesn't this import anything?
use abc_parser::datatypes::*;
use svg::Document;
use svg::Node;
use svg::node::element::{Circle, Line, Text};

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

/*
* Notes on spacing:
*   - notes are grouped closer or farther apart in order to accomodate a certain number of measures
*     on a line.
*   - a note takes up space proportional to its value, e.g. a half note takes up the space of two
*     quarter notes.
*   - it seems like there is a certain uniform spacing between the end of a measure and the start
*     of a note. However, that spacing varies based on who is making the sheet music. I notice that
*     thesession.org (which I believe uses old abcjs), has a particularly long default distance.
*   - OH huge breakthrough is that in ABC notation, newlines matter! So we don't have to think
*     about how many bars to fit on the page, that is decided for us.
*   - there's a constraint on how close the notes can be. they can't ever touch. there must be a
*     point at which an error is thrown or the width is widened.
*   - https://www.abcjs.net/abcjs-editor is useful for testing out these things
*
*
*   In order to know the position of a note, we have to go from the tokenized form down into the
*   specific. We want to know how many notes are trying to fit on this system. Then we can assign
*   space to each measure equally. BUT this is not actually true, because each accidental will
*   shift over everytghing else to make room for it.
*
*   Not sure how this intersects with size constraints. But that can be up to the user to figure
*   out. We'll generate a document and it will have a size. You can adjust parameters to adjust
*   that size.
*
* As claude recommended me long ago, I think we can go at this in two passes.
* One part is just to know how to render each musical element. What combination of notes and stems
* and such. The other part is knowing how close together they should be.
*
* This breaks down slightly with drawing the note stems and the lines above them. They kinda need
* to know.
*
X: 1
T: Banish Misfortune
R: jig
M: 6/8
L: 1/8
K: Dmix
|:fed cAG|A2d cAG|F2D DED|FEF GFG| AGA cAG|AGA cde|fed cAG|Ad^c d3:| |:f2d d^cd|f2g agf|e2c cBc|e2f gfe| f2g agf|e2f gfe|fed cAG|Ad^c d3:| |:f2g e2f|d2e c2d|ABA GAG|F2F GED| c3 cAG|AGA cde|fed cAG|Ad^c d3:|

*/

pub struct LayoutConfig {
    pub file_name: String,
    pub margin_left: f32,
    pub margin_top: f32,
}

enum StemType {
    Up,
    Down,
}

#[derive(Debug)]
struct RendMeasure {
    symbols: Vec<RendSymbol>,
}

#[derive(Debug)]
struct RendLine {
    // TODO: add prefix_symbols
    //
    // Each measure is "symbol suffixed", it is responsible for drawing its closing symbol.
    // The starting symbol of the first measure is handled by the prefix_symbols section of the
    // line.
    measures: Vec<RendMeasure>,
    total_weight: Option<f32>,
}

// TODO: wrap everything in one of these to make it easy to modify position in multiple passes
#[derive(Debug)]
struct RendSymbol {
    x: f32,
    y: f32,
    symbol: MusicSymbol,
}

pub fn render_abc(abc_str: &str, config: LayoutConfig) -> svg::Document {
    let mut tune_book = match abc::tune_book(abc_str) {
        Ok(tb) => tb,
        Err(error) => panic!("Problem parsing tune book: {error}"),
    };

    // TODO: tolerate multiple tunes
    let mut tune = tune_book.tunes.remove(0);
    let body = tune.body.take().expect("No tune body");
    let header = tune.header;

    // Determine this.. somehow.
    // TODO: add this to the LayoutConfig
    let available_width = 30.0;

    // Calculate width
    let mut min_space_needed = 0.0;
    // TODO: figure out how to get name from header for error reporting

    // Alright, we've got our available space.
    // Now we can divide it into measures.
    // At first we can ignore the existance of accidentals, though we'll have to think about it
    // some day.
    let mut measures: Vec<RendMeasure> = Vec::new();
    let mut nodes: Vec<Box<dyn Node>> = Vec::new();

    let mut lines: Vec<RendLine> = Vec::new();

    for abc_line in body.music {
        let mut line = RendLine {
            measures: Vec::new(),
            total_weight: None,
        };
        let mut measure_symbols: Vec<RendSymbol> = Vec::new();
        let mut total_weight: f32 = 0.0;

        // For now we can use the x of RendSymbol to represent the number of "units" from the left
        // the note should be, not in terms of base unit, but in terms of the time unit of the
        // note.

        // add 3 note widths for g clef
        total_weight += 3.0;

        // Handle clef
        for symbol in abc_line.symbols {
            match symbol {
                MusicSymbol::Note {
                    decoration: _,
                    accidental: _,
                    note: _,
                    octave: _,
                    length,
                } => {
                    let mut symbol = symbol;
                    measure_symbols.push(RendSymbol {
                        x: total_weight,
                        y: 0.0,
                        symbol: symbol,
                    });
                    total_weight += length;
                }
                MusicSymbol::Bar(_) => {
                    measure_symbols.push(RendSymbol {
                        x: total_weight,
                        y: 0.0,
                        symbol: symbol,
                    });
                    total_weight += 1.0;
                    line.measures.push(RendMeasure {
                        symbols: std::mem::take(&mut measure_symbols),
                    });
                }
                MusicSymbol::VisualBreak() => {
                    measure_symbols.push(RendSymbol {
                        x: total_weight,
                        y: 0.0,
                        symbol: symbol,
                    });
                }
                _ => {
                    measure_symbols.push(RendSymbol {
                        x: 0.0,
                        y: 0.0,
                        symbol: symbol,
                    });
                }
            }
        }

        println!("Total weight: {}", total_weight);
        println!(
            "Each unit gets {} actual space",
            available_width / total_weight
        );

        line.total_weight = Some(total_weight);
        lines.push(line);
    }

    for l in lines {
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

        for m in l.measures {
            push_svg_vec(
                &mut nodes,
                render_measure(
                    m,
                    &config,
                    available_width / l.total_weight.expect("Total weight must be set to render"),
                    2.0,
                ),
            );
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

// X is in terms of note-length-units. We need to no conversion
fn render_measure(
    measure: RendMeasure,
    config: &LayoutConfig,
    base_unit_conversion_factor: f32,
    note_length_factor: f32, // add to note length. e.g. -0.5 means a length of 1 is actually a
                             // length of 0.5
) -> Vec<Box<dyn Node>> {
    let mut nodes: Vec<Box<dyn Node>> = Vec::new();

    for sym in measure.symbols {
        match sym.symbol {
            MusicSymbol::Note {
                decoration: _,
                accidental: _,
                note,
                octave,
                length,
            } => {
                let x = config.margin_left + sym.x * BASE_UNIT * base_unit_conversion_factor;

                let adjusted_length = length / note_length_factor;

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

                let y = config.margin_top + 4.0 * BASE_UNIT - note_offset as f32 * BASE_UNIT / 2.0
                    + 8.0 * (octave - 1) as f32 * BASE_UNIT;

                push_svg_vec(&mut nodes, stem_draw(x, y, StemType::Down, adjusted_length));

                // Draw note head
                if adjusted_length > 0.0 && adjusted_length < 2.0 {
                    nodes.push(text_node_create('\u{E0A4}', x, y));
                } else if adjusted_length == 2.0 {
                    nodes.push(text_node_create('\u{E0A3}', x, y));
                } else if adjusted_length == 4.0 {
                    nodes.push(text_node_create('\u{E0A2}', x, y));
                }
            }
            MusicSymbol::Bar(bar_string) => {
                let x = config.margin_left + sym.x * BASE_UNIT * base_unit_conversion_factor;
                let y = config.margin_top + 4.0 * BASE_UNIT;
                if bar_string == "|" {
                    nodes.push(text_node_create('\u{E030}', x, y));
                } else if bar_string == "|:" {
                    nodes.push(text_node_create('\u{E040}', x, y));
                } else if bar_string == ":|" {
                    nodes.push(text_node_create('\u{E041}', x, y));
                }
            }
            _ => {
                //println!("Not handling");
                //dbg!(sym.symbol);
            }
        };
    }

    return nodes;
}

// Returns base units of horizontal space required for a certain symbol
fn required_hspace(sym: MusicSymbol) -> f32 {
    match sym {
        MusicSymbol::Note {
            decoration: _,
            accidental: _,
            note: _,
            octave: _,
            length,
        } => length.sqrt(),
        _ => 0.0,
    }
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
