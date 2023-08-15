/// Levels of segments in a LaTeX document.
/// A Node has the level of the next expected segment in the subtree beneath it including itself.
pub(crate) const LEVELS: [(i8, &str); 7] = [
    (-1, "part"),
    (0, "chapter"),
    (1, "section"),
    (2, "subsection"),
    (3, "subsubsection"),
    (4, "paragraph"),
    (5, "subparagraph"),
];
pub(crate) const LEAF_LEVEL: i8 = 6;

pub(crate) const UNCOUNTED_SEGMENT_MARKER: &str = "*";
