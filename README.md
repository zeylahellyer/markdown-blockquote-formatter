# markdown-blockquote-formatter

Format text as a markdown blockquote, with smart soft and hard blockquote
and line limits and other nifty configuration options.

This library is `no_std` and only uses `core`.

### Examples

With a soft limit that would cut the blockquote off in the middle of the
word "nifty" in the input were it not for the hard limit,
format the previous message split across two paragraphs:

```rust
use markdown_blockquote_formatter::Blockquote;

const INPUT: &str = "Format text as a markdown blockquote.

Configuration includes soft and hard blockquote and line limits and other
nifty configuration options.";

const OUTPUT: &str = "> Format text as a markdown blockquote.
>\x20
> Configuration includes soft and hard blockquote…";

// A soft limit of 81 cuts us off in the middle of the word "blockquote",
// but by specifying a hard limit we can continue the blockquote until the
// end of the word.
let blockquote = Blockquote::new(INPUT).soft_limit(81).hard_limit(10);

assert_eq!(blockquote.to_string(), OUTPUT);
```
