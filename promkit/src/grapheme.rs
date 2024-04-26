use std::{
    collections::VecDeque,
    fmt,
    ops::{Deref, DerefMut},
};

use unicode_width::UnicodeWidthChar;

use crate::crossterm::style::{Attribute, ContentStyle};

/// Represents a single grapheme (character) with its display width and optional styling.
///
/// This structure is similar to `Grapheme` but includes styling information directly.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StyledGrapheme {
    ch: char,
    width: usize,
    style: ContentStyle,
}

impl From<char> for StyledGrapheme {
    fn from(ch: char) -> Self {
        Self {
            ch,
            width: UnicodeWidthChar::width(ch).unwrap_or(0),
            style: ContentStyle::default(),
        }
    }
}

impl ToString for StyledGrapheme {
    fn to_string(&self) -> String {
        self.ch.to_string()
    }
}

impl StyledGrapheme {
    pub fn new(ch: char, style: ContentStyle) -> Self {
        Self {
            ch,
            width: UnicodeWidthChar::width(ch).unwrap_or(0),
            style,
        }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn apply_style(&mut self, style: ContentStyle) {
        self.style = style;
    }
}

/// A collection of `StyledGrapheme` instances.
///
/// This structure supports operations like calculating the total display width of the collection
/// and generating a display representation that respects the applied styles.
#[derive(Clone, Default, PartialEq, Eq)]
pub struct StyledGraphemes(pub VecDeque<StyledGrapheme>);

impl Deref for StyledGraphemes {
    type Target = VecDeque<StyledGrapheme>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for StyledGraphemes {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl FromIterator<StyledGraphemes> for StyledGraphemes {
    fn from_iter<I: IntoIterator<Item = StyledGraphemes>>(iter: I) -> Self {
        let concatenated = iter
            .into_iter()
            .flat_map(|g| g.0.into_iter())
            .collect::<VecDeque<StyledGrapheme>>();
        StyledGraphemes(concatenated)
    }
}

impl FromIterator<StyledGrapheme> for StyledGraphemes {
    fn from_iter<I: IntoIterator<Item = StyledGrapheme>>(iter: I) -> Self {
        let mut g = StyledGraphemes::default();
        for i in iter {
            g.push_back(i);
        }
        g
    }
}

impl<S: AsRef<str>> From<S> for StyledGraphemes {
    fn from(string: S) -> Self {
        Self::from_str(string, ContentStyle::default())
    }
}

impl fmt::Debug for StyledGraphemes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for styled_grapheme in self.iter() {
            write!(f, "{}", styled_grapheme.ch)?;
        }
        Ok(())
    }
}

impl ToString for StyledGraphemes {
    fn to_string(&self) -> String {
        self.iter().map(|g| g.ch).collect()
    }
}

impl StyledGraphemes {
    pub fn from_str<S: AsRef<str>>(string: S, style: ContentStyle) -> Self {
        string
            .as_ref()
            .chars()
            .map(|ch| StyledGrapheme::new(ch, style))
            .collect()
    }

    /// Returns a `Vec<char>` containing the characters of all `Grapheme` instances in the collection.
    pub fn chars(&self) -> Vec<char> {
        self.0.iter().map(|grapheme| grapheme.ch).collect()
    }

    /// Calculates the total display width of all `Grapheme` instances in the collection.
    pub fn widths(&self) -> usize {
        self.0.iter().map(|grapheme| grapheme.width).sum()
    }

    /// Replaces the specified range with the given string.
    pub fn replace_range<S: AsRef<str>>(&mut self, range: std::ops::Range<usize>, replacement: S) {
        // Remove the specified range.
        for _ in range.clone() {
            self.0.remove(range.start);
        }

        // Insert the replacement at the start of the range.
        let replacement_graphemes: StyledGraphemes = replacement.as_ref().into();
        for grapheme in replacement_graphemes.0.iter().rev() {
            self.0.insert(range.start, grapheme.clone());
        }
    }

    /// Applies a given style to all `StyledGrapheme` instances within the collection.
    pub fn apply_style(mut self, style: ContentStyle) -> Self {
        for grapheme in &mut self.0 {
            grapheme.apply_style(style);
        }
        self
    }

    /// Applies a given style to a specific `StyledGrapheme` at the specified index.
    pub fn apply_style_at(mut self, idx: usize, style: ContentStyle) -> Self {
        if let Some(grapheme) = self.0.get_mut(idx) {
            grapheme.apply_style(style);
        }
        self
    }

    /// Applies a given attribute to all `StyledGrapheme` instances within the collection.
    pub fn apply_attribute(mut self, attr: Attribute) -> Self {
        for styled_grapheme in &mut self.0 {
            styled_grapheme.style.attributes.set(attr);
        }
        self
    }

    /// Returns a displayable format of the styled graphemes.
    pub fn styled_display(&self) -> StyledGraphemesDisplay<'_> {
        StyledGraphemesDisplay {
            styled_graphemes: self,
        }
    }

    /// Organizes the `StyledGraphemes` into a matrix format based on specified width and height,
    /// considering an offset for pagination or scrolling.
    pub fn matrixify(
        &self,
        width: usize,
        height: usize,
        offset: usize,
    ) -> (Vec<StyledGraphemes>, usize) {
        let mut all = vec![];
        let mut row = StyledGraphemes::default();
        for styled in self.iter() {
            let width_with_next_char = row.iter().fold(0, |mut layout, g| {
                layout += g.width;
                layout
            }) + styled.width;
            if !row.is_empty() && width < width_with_next_char {
                all.push(row);
                row = StyledGraphemes::default();
            }
            if width >= styled.width {
                row.push_back(styled.clone());
            }
        }
        if !row.is_empty() {
            all.push(row);
        }

        if all.is_empty() {
            return (vec![], 0);
        }

        let chunks: Vec<Vec<StyledGraphemes>> =
            all.chunks(height).map(|chunk| chunk.to_vec()).collect();

        let chunk_index = std::cmp::min(offset / height, chunks.len().saturating_sub(1));
        let selected_chunk = chunks.get(chunk_index).cloned().unwrap_or_default();

        let local_offset = offset % height;

        (selected_chunk, local_offset)
    }

    /// Truncates the collection of `StyledGraphemes` to fit within a specified width.
    pub fn truncate_to_width(&self, width: usize) -> StyledGraphemes {
        let mut row = StyledGraphemes::default();
        for ch in self.iter() {
            let width_with_next_char = row.iter().fold(0, |mut layout, g| {
                layout += g.width;
                layout
            }) + ch.width;
            if width < width_with_next_char {
                break;
            }
            if width >= ch.width {
                row.push_back(ch.clone());
            }
        }
        row
    }
}

pub struct StyledGraphemesDisplay<'a> {
    styled_graphemes: &'a StyledGraphemes,
}

impl<'a> fmt::Display for StyledGraphemesDisplay<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for styled_grapheme in self.styled_graphemes.iter() {
            write!(f, "{}", styled_grapheme.style.apply(styled_grapheme.ch))?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    mod from_str {
        use super::*;

        #[test]
        fn test() {
            let style = ContentStyle::default();
            let graphemes = StyledGraphemes::from_str("abc", style.clone());
            assert_eq!(3, graphemes.0.len());
            assert!(graphemes.0.iter().all(|g| g.style == style));
        }
    }

    mod chars {
        use super::*;

        #[test]
        fn test() {
            let graphemes = StyledGraphemes::from("abc");
            let chars = graphemes.chars();
            assert_eq!(vec!['a', 'b', 'c'], chars);
        }
    }

    mod widths {
        use super::*;

        #[test]
        fn test() {
            let graphemes = StyledGraphemes::from("a b");
            assert_eq!(3, graphemes.widths()); // 'a' and 'b' are each 1 width, and space is 1 width
        }
    }

    mod replace_range {
        use super::*;

        #[test]
        fn test() {
            let mut graphemes = StyledGraphemes::from("Hello");
            graphemes.replace_range(1..5, "i");
            assert_eq!("Hi", graphemes.to_string());
        }
    }

    mod apply_style {
        use crate::{crossterm::style::Color, style::StyleBuilder};

        use super::*;

        #[test]
        fn test_apply_style_to_all() {
            let mut graphemes = StyledGraphemes::from("abc");
            let new_style = StyleBuilder::new().fgc(Color::Green).build();
            graphemes = graphemes.apply_style(new_style.clone());
            assert!(graphemes.iter().all(|g| g.style == new_style));
        }
    }

    mod apply_style_at {
        use crate::{crossterm::style::Color, style::StyleBuilder};

        use super::*;

        #[test]
        fn test_apply_style_at_specific_index() {
            let mut graphemes = StyledGraphemes::from("abc");
            let new_style = StyleBuilder::new().fgc(Color::Green).build();
            graphemes = graphemes.apply_style_at(1, new_style.clone());
            assert_eq!(graphemes.0[1].style, new_style);
            assert_ne!(graphemes.0[0].style, new_style);
            assert_ne!(graphemes.0[2].style, new_style);
        }

        #[test]
        fn test_apply_style_at_out_of_bounds_index() {
            let mut graphemes = StyledGraphemes::from("abc");
            let new_style = StyleBuilder::new().fgc(Color::Green).build();
            graphemes = graphemes.apply_style_at(5, new_style.clone()); // Out of bounds
            assert_eq!(graphemes.0.len(), 3); // Ensure no changes in length
        }
    }

    mod apply_attribute {
        use super::*;

        #[test]
        fn test_apply_attribute_to_all() {
            let mut graphemes = StyledGraphemes::from("abc");
            graphemes = graphemes.apply_attribute(Attribute::Bold);
            assert!(graphemes
                .iter()
                .all(|g| g.style.attributes.has(Attribute::Bold)));
        }
    }

    mod styled_display {
        use super::*;

        #[test]
        fn test_styled_display_output() {
            let graphemes = StyledGraphemes::from("abc");
            let display = graphemes.styled_display();
            assert_eq!(format!("{}", display), "abc"); // Assuming default styles do not alter appearance
        }
    }

    mod matrixify {
        use super::*;

        #[test]
        fn test_with_single_line_no_offset() {
            let input =
                StyledGraphemes::from("Hello, world! This is a longer test without offset.");
            let (matrix, offset) = input.matrixify(50, 1, 0);
            assert_eq!(1, matrix.len());
            assert_eq!(
                "Hello, world! This is a longer test without offset",
                matrix[0].to_string()
            );
            assert_eq!(0, offset);
        }

        #[test]
        fn test_with_multiple_lines_and_offset() {
            let input = StyledGraphemes::from("One Two Three Four Five Six Seven Eight Nine Ten");
            let (matrix, offset) = input.matrixify(10, 3, 10);
            assert_eq!(2, matrix.len());
            assert_eq!("ven Eight ", matrix[0].to_string());
            assert_eq!("Nine Ten", matrix[1].to_string());
            assert_eq!(1, offset);
        }

        #[test]
        fn test_with_empty_input() {
            let input = StyledGraphemes::default();
            let (matrix, offset) = input.matrixify(10, 2, 0);
            assert!(matrix.is_empty());
            assert_eq!(0, offset);
        }

        #[test]
        fn test_with_large_offset_beyond_content() {
            let input = StyledGraphemes::from("Short text");
            let (matrix, offset) = input.matrixify(10, 2, 20);
            assert_eq!(1, matrix.len());
            assert_eq!("Short text", matrix[0].to_string());
            assert_eq!(0, offset);
        }
    }

    mod truncate_to_width {
        use super::*;

        #[test]
        fn test() {
            assert_eq!(
                StyledGraphemes::from(">> a"),
                StyledGraphemes::from(">> aaa ").truncate_to_width(4)
            );
        }

        #[test]
        fn test_with_emoji() {
            assert_eq!(
                StyledGraphemes::from("😎"),
                StyledGraphemes::from("😎").truncate_to_width(2)
            );
        }

        #[test]
        fn test_with_emoji_at_narrow_terminal() {
            assert_eq!(
                StyledGraphemes::from(""),
                StyledGraphemes::from("😎").truncate_to_width(1)
            );
        }
    }
}
