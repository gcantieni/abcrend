use abcrend::{self, LayoutConfig};

fn main() {
    // TODO: solve whatever issue is causing cooley's to not parse.
    // See https://gitlab.com/Askaholic/rust-abc-2/-/issues/5
    // TODO: I think rust-abc-2 doesn't handle L 1/8
    let _cooleys = String::from(
        "X: 1
T: Cooley's
R: reel
M: 4/4
L: 1/8
K: Edor
|:D2|EBBA B2 EB|B2 AB dBAG:|
",
    );

    // scale
    let _major_scale = "M:4/4
O:Irish
R:Reel

X:1
T:Untitled Reel
C:Trad.
K:D
|C/2 D4 E F G A B c|";

    let data = "M:4/4
O:Irish
R:Reel

X:1
T:Untitled Reel
C:Trad.
K:D
eg|a2ab ageg|agbg agef:|";

    let c = LayoutConfig {
        file_name: String::from("example.svg"),
        margin_left: 30.0,
        margin_top: 30.0,
    };

    abcrend::render_abc(&_major_scale, c);
}

// Someday do the real thing
/*
"X: 1
T: Cooley's
R: reel
M: 4/4
L: 1/8
K: Edor
|:D2|EBBA B2 EB|B2 AB dBAG|FDAD BDAD|FDAD dAFD|
EBBA B2 EB|B2 AB defg|afec dBAF|DEFD E2:|
|:gf|eB B2 efge|eB B2 gedB|A2 FA DAFA|A2 FA defg|
eB B2 eBgB|eB B2 defg|afec dBAF|DEFD E2:|",
*/
