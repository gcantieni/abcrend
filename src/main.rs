use abcrend::{self, LayoutConfig};

fn main() {
    // TODO: solve whatever issue is causing cooley's to not parse.
    // See https://gitlab.com/Askaholic/rust-abc-2/-/issues/5
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

    // Try it with the one from testing:
    let data = "M:4/4
O:Irish
R:Reel

X:1
T:Untitled Reel
C:Trad.
K:D
eg|a2ab ageg|agbg agef|f2d2 d2:|";

    let c = LayoutConfig {
        file_name: String::from("example.svg"),
        margin_left: 30.0,
        margin_top: 30.0,
    };

    abcrend::render_abc(&data, c);
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
