#![doc = include_str!("../README.md")]
#![no_std]

#[cfg(test)]
extern crate alloc;

use core::fmt::{Display, Error as FmtError, Formatter};

/// String for starting a blockquote line.
const BLOCKQUOTE_LINE: &str = "> ";

/// Character for an ellipsis.
const ELLIPSIS: char = '…';

/// Character for a newline.
const NEWLINE: char = '\n';

/// Quote some text in a markdown blockquote.
///
/// # Examples
///
/// Quote the text "hey, this is cool!":
///
/// ```
/// use markdown_blockquote_formatter::Blockquote;
///
/// let blockquote = Blockquote::new("hey, this is cool!");
///
/// assert_eq!(blockquote.to_string(), "> hey, this is cool!");
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Blockquote<'a> {
    hard_limit: Option<usize>,
    soft_limit: usize,
    text: &'a str,
    with_ellipsis: bool,
}

impl<'a> Blockquote<'a> {
    /// Create a new markdown blockquote formatter.
    pub const fn new(text: &'a str) -> Self {
        Self {
            hard_limit: None,
            soft_limit: usize::MAX,
            text,
            with_ellipsis: true,
        }
    }

    /// There is no soft limit in practice by default.
    pub const fn soft_limit(mut self, soft_limit: usize) -> Self {
        self.soft_limit = soft_limit;

        self
    }

    /// Set the hard limit to break off the formatted text.
    ///
    /// The hard limit is *in addition to* the soft limit. Providing a value of
    /// 50 via [`soft_limit`] and 10 via [`hard_limit`] results in a hard limit
    /// of 60, saturating at [`usize::MAX`].
    ///
    /// There is no hard limit by default.
    ///
    /// [`hard_limit`]: Self::hard_limit
    /// [`soft_limit`]: Self::soft_limit
    pub const fn hard_limit(mut self, hard_limit: usize) -> Self {
        self.hard_limit = Some(hard_limit);

        self
    }

    /// Whether to include ellipsis upon reaching the end of the formatting.
    ///
    /// Ellipsis are included by default.
    pub const fn with_ellipsis(mut self, with_ellipsis: bool) -> Self {
        self.with_ellipsis = with_ellipsis;

        self
    }

    /// Whether the blockquote will be empty upon formatting.
    ///
    /// This will be the case if the input text is empty or only consists of
    /// whitespace.
    ///
    /// Blockquotes will short circuit and format nothing when empty.
    pub fn is_empty(&self) -> bool {
        self.text.is_empty() || self.text.trim().is_empty()
    }

    fn reached_limit(&self, index: usize, soft: bool) -> bool {
        let limit = if soft {
            self.soft_limit
        } else {
            let hard_limit = self.hard_limit.unwrap_or_default();

            self.soft_limit.saturating_add(hard_limit)
        };

        index >= limit
    }

    fn remaining_empty(&self, index: usize) -> bool {
        self.text
            .get(index..)
            .map(|remaining| remaining.trim_end().is_empty())
            .unwrap_or_default()
    }
}

impl Display for Blockquote<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        #[derive(Clone, Copy, Eq, PartialEq)]
        enum Stage {
            Ongoing,
            StartLine,
        }

        if self.is_empty() {
            return Ok(());
        }

        let chars = self.text.chars();
        let mut index = 0;
        let mut stage = Stage::StartLine;

        for character in chars {
            // Stop if all the remaining text is whitespace.
            if let Some(text_slice) = self.text.get(index..) {
                if text_slice.trim_end().is_empty() {
                    break;
                }
            }

            if stage == Stage::StartLine {
                f.write_str(BLOCKQUOTE_LINE)?;

                if character != NEWLINE {
                    stage = Stage::Ongoing;
                }
            }

            if self.reached_limit(index, character.is_whitespace()) {
                break;
            }

            write_char(character, f)?;

            index += 1;

            if character == NEWLINE {
                stage = Stage::StartLine;
            }
        }

        if self.with_ellipsis && !self.remaining_empty(index) {
            write_char(ELLIPSIS, f)?;
        }

        Ok(())
    }
}

fn write_char(character: char, f: &mut Formatter<'_>) -> Result<(), FmtError> {
    let mut buf = [0u8; 4];
    let string_slice = character.encode_utf8(&mut buf);

    f.write_str(string_slice)
}

#[cfg(test)]
mod tests {
    use super::Blockquote;
    use alloc::{borrow::ToOwned, fmt::Debug, string::ToString};
    use static_assertions::assert_impl_all;

    assert_impl_all!(Blockquote: Debug, Send, Sync);

    #[test]
    fn test_simple() {
        const INPUT: &str = "this is a simple test";
        const OUTPUT: &str = "> this is a simple test";

        let formatter = Blockquote::new(INPUT).soft_limit(50);

        assert_eq!(formatter.to_string(), OUTPUT);
    }

    #[test]
    fn test_is_empty() {
        assert!(Blockquote::new("").is_empty());
        assert!(Blockquote::new(" \n  \t ").is_empty());
    }

    #[test]
    fn test_newlines() {
        const EXPECTED: &str = "> test\n> two\n> three";

        let formatter = Blockquote::new("test\ntwo\nthree").soft_limit(50);

        assert_eq!(formatter.to_string(), EXPECTED);
    }

    #[test]
    fn test_soft_limit() {
        const EXPECTED: &str = "> this is just:\n> a really cool…";

        let before_cut = "this is just:\na really co";
        let after_cut = "ol test!";
        let text = before_cut.to_owned() + after_cut;

        let formatter = Blockquote::new(&text)
            .soft_limit(before_cut.len())
            .hard_limit(10);

        assert_eq!(formatter.to_string(), EXPECTED);
    }

    #[test]
    fn test_soft_limit_cutoff() {
        const EXPECTED: &str = "> this is just:\n> a really coo…";

        let before_cut = "this is just:\na really co";
        let after_cut = "ol test!";
        let text = before_cut.to_owned() + after_cut;

        let formatter = Blockquote::new(&text)
            .soft_limit(before_cut.len())
            .hard_limit(1);

        assert_eq!(formatter.to_string(), EXPECTED);
    }

    #[test]
    fn test_soft_limit_none() {
        const INPUT: &str = "this text is too long :(";
        const OUTPUT: &str = "> this text is too lo…";

        let formatter = Blockquote::new(INPUT).soft_limit(19);

        assert_eq!(formatter.to_string(), OUTPUT);
    }

    #[test]
    fn test_soft_limit_zero() {
        const INPUT: &str = "this text is too long :(";
        const OUTPUT: &str = "> this text is too lo…";

        let formatter = Blockquote::new(INPUT).soft_limit(19);

        assert_eq!(formatter.to_string(), OUTPUT);
    }

    #[test]
    fn test_end_with_newline() {
        const INPUT: &str = "test\nagain\n\n";
        const OUTPUT: &str = "> test\n> again";

        let formatter = Blockquote::new(INPUT);
        assert_eq!(formatter.to_string(), OUTPUT);
    }
}
