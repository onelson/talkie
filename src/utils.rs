/// Given some amount of time, use the rate to determine how much of the time
/// went unused and how many glyphs should now be revealed.
pub fn calc_glyphs_to_reveal(delta_secs: f32, glyphs_per_sec: f32) -> (usize, f32) {
    let reveal_how_many = (delta_secs * glyphs_per_sec).trunc();
    let remainder = delta_secs - (reveal_how_many / glyphs_per_sec);
    (reveal_how_many as usize, remainder)
}

#[cfg(test)]
mod tests {
    use super::calc_glyphs_to_reveal;
    use assert_approx_eq::assert_approx_eq;

    /// If the delta is not big enough to reveal at least one glyph, then the
    /// remainder should be the entire delta.
    #[test]
    fn test_delta_carries_over() {
        let (count, remainder) = calc_glyphs_to_reveal(1.0, 0.5);
        assert_eq!(0, count);
        assert_approx_eq!(1.0, remainder);
    }

    #[test]
    fn test_delta_zero_when_glyph_revealed() {
        let (count, remainder) = calc_glyphs_to_reveal(2.0, 0.5);
        assert_eq!(1, count);
        assert_approx_eq!(0.0, remainder);
    }

    #[test]
    fn test_delta_remainder_when_glyph_revealed() {
        let (count, remainder) = calc_glyphs_to_reveal(2.2, 0.5);
        assert_eq!(1, count);
        assert_approx_eq!(0.2, remainder);
    }

    #[test]
    fn test_multi_glyph_remainder() {
        let (count, remainder) = calc_glyphs_to_reveal(5.2, 2.0);
        assert_eq!(10, count);
        assert_approx_eq!(0.2, remainder);
    }
}
