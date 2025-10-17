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
    space: f32,
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

    for line in body.music {
        // This paper gives some interesting ideas: https://drive.google.com/file/d/1ztVuZrLYH0eUludsiY3jMS4GbxbPi-BB/view
        // The central idea is to use a priority queue to improve efficiency.
        // It is very performance focused. I'm not fully convinced.
        //
        // I think I need a naive implementation first so I can rewrite it better after.

        // Position each symbol where it needs to be to give it minimum space it needs to avoid
        // collisions.
        // Distribute available space to the remaining symbols proportional to weight of Symbol.

        // What's the dumbest thing I could do? I split the avialable space proportional to the
        // note weight. Assign each note a weight. Sum up the total weight of all notes. Divide the
        // space by the overall weight to get the "conversion rate" of note weight to note space.
        // That will give me my X value.

        let mut measure_symbols: Vec<RendSymbol> = Vec::new();
        let mut measure_space: f32 = 0.0;
        let mut total_weight: f32 = 0.0;

        // For now we can use the x of RendSymbol to represent the number of "units" from the left
        // the note should be, not in terms of base unit, but in terms of the time unit of the
        // note.

        for symbol in line.symbols {
            //dbg!(&symbol);
            match symbol {
                MusicSymbol::Note {
                    decoration: _,
                    accidental: _,
                    note: _,
                    octave: _,
                    length,
                } => {
                    //measure_space += 1.0;
                    total_weight += length;
                    measure_symbols.push(RendSymbol {
                        x: total_weight,
                        y: 0.0,
                        symbol: symbol,
                    });
                }
                MusicSymbol::Bar(_) => {
                    total_weight += 1.0;
                    measure_symbols.push(RendSymbol {
                        x: total_weight,
                        y: 0.0,
                        symbol: symbol,
                    });

                    //measures.push(RendMeasure {
                    //    symbols: std::mem::take(&mut measure_symbols),
                    //    space: measure_space,
                    //});
                }
                MusicSymbol::VisualBreak() => {
                    total_weight += 1.0;
                    measure_symbols.push(RendSymbol {
                        x: total_weight,
                        y: 0.0,
                        symbol: symbol,
                    });
                }
                _ => {
                    dbg!(&symbol);
                    measure_symbols.push(RendSymbol {
                        x: 0.0,
                        y: 0.0,
                        symbol: symbol,
                    });
                }
            }
        }

        // Add up space required for line.
        //let line_space = measures.iter().fold(0.0, |acc, m| acc + m.space);
        //println!("Needed space: {}", line_space);
        println!("Total weight: {}", total_weight);
        println!(
            "Each unit gets {} actual space",
            available_width / total_weight
        );

        //if line_space > available_width {
        //    panic!(
        //        "Available space is less than space needed by maximim line. Either reduce line size or increase available size.",
        //    );
        //}
    }

    println!("Determined minimum px needed {}", min_space_needed);

    let mut doc = Document::new()
        .set("viewBox", (0, 0, 300, 300))
        .set("font-family", "Bravura");

    for n in nodes {
        doc = doc.add(n);
    }

    svg::save("example.svg", &doc).unwrap();

    return doc;
}

fn _draw_experimental_staff(config: LayoutConfig) -> svg::Document {
    // Draw staff
    let mut nodes: Vec<Box<dyn Node>> = Vec::new();

    // The gclef records its position based on the mid-point of its back.
    // Thus it can be aligned with the lines of the staff and it looks about right.
    let gclef = text_node_create(
        '\u{E050}',
        config.margin_left,
        config.margin_top + 3.0 * BASE_UNIT,
    );

    // The actual lines, seaparated by the width of a note
    let line_stroke_width = 0.1 * BASE_UNIT;
    let line_length = BASE_UNIT * 30.0;
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

    nodes.push(gclef);

    // Draw note

    // quote:
    // Note stem direction changes based on where the note lies on the musical staff.
    // If the note head is on the middle line or above, the stem goes upside down.
    // If the notehead lies on the second space from the bottom or down, the stem goes up.

    //nodes.push(ledger_line);
    // nodes.push(debug_draw_dot(first_note_x, first_note_y, 0.1));

    let first_note_x = config.margin_left + 5.0 * BASE_UNIT;
    let first_note_y = config.margin_top + 5.0 * BASE_UNIT;
    // Let's do the rest of the notes in the scale
    for i in 1..8 {
        let f = i as f32;
        let nx = first_note_x + f * BASE_UNIT + f * BASE_UNIT; // Seems like note spacing is
        // usually around two notes over
        let ny = first_note_y - f * BASE_UNIT * 0.5;

        nodes.push(text_node_create('\u{E0A4}', nx, ny));

        let stem_type = if i < 6 { StemType::Up } else { StemType::Down };

        nodes.push(stem_draw(nx, ny, stem_type));
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
fn render_sym(
    sym: RendSymbol,
    config: LayoutConfig,
    base_unit_conversion_factor: f32,
) -> Vec<Box<dyn Node>> {
    let mut nodes: Vec<Box<dyn Node>> = Vec::new();

    let x = config.margin_left + sym.x * BASE_UNIT * base_unit_conversion_factor;
    let y = config.margin_top + 5.0 * BASE_UNIT;

    nodes.push(stem_draw(x, y, StemType::Up));
    nodes.push(text_node_create('\u{E0A4}', x, y));

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
fn stem_draw(note_x: f32, note_y: f32, t: StemType) -> Box<dyn Node> {
    match t {
        StemType::Up => text_node_create(
            '\u{E210}',
            note_x + (BASE_UNIT * 1.11), // Approx note + 1
            note_y - 0.1 * BASE_UNIT,
        ),
        StemType::Down => text_node_create(
            '\u{E210}',
            note_x + 0.06 * BASE_UNIT,
            note_y + 3.60 * BASE_UNIT, // A stem is approx 3 notes high
        ),
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
