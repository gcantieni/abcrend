//! ABC music notation to SVG renderer
//! 
//! This crate will convert ABC notation to SVG musical scores.

/// Placeholder function
pub fn placeholder() -> &'static str {
    "abcrend - coming soon!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        assert_eq!(placeholder(), "abcrend - coming soon!");
    }
}
