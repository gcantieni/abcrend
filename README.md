
# abcrend

**ABC music notation to SVG renderer**

*Work in progress - coming soon!*

This crate will provide native Rust parsing and rendering of ABC notation to SVG format.

Note that there are multiple options for how font rendering could be handled.
Currently, I installed the included fonts globally. We could explore options
of embedding the font within the svg if needs be.

## Planned Features
- ABC 2.1 standard support
- High-quality SVG output
- Customizable rendering options
- No external dependencies for rendering

## Roadmap
- Render staff and cleff at fixed width
- Determine y coordinate for each note based on dimension
- Use beruva to render render note heads.
- Render notes with no stems or accidentals
- Flag
- Beaming, grouping eighth notes/sixtheenth notes together

## License
MIT OR Apache-2.0

