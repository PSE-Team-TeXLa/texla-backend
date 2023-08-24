//! `latex_constants` defines some tokens used in Latex to as delimiters or keywords.
// LaTeX files and paths
pub const LATEX_FILE_EXTENSION: &str = "tex";
pub const LATEX_PATH_SEPARATOR: &str = "/";

// commands
pub(crate) const BEGIN: &str = "\\begin";
pub(crate) const END: &str = "\\end";
pub const INPUT: &str = "\\input";
// pub' instead of 'pub(crate)' used since it's needed in the other crate
pub(crate) const INCLUDEGRAPHICS: &str = "\\includegraphics";
pub(crate) const CAPTION: &str = "\\caption";
pub(crate) const LABEL: &str = "\\label";

// environments
pub(crate) const DOCUMENT_BEGIN: &str = "\\begin{document}";
pub(crate) const DOCUMENT_END: &str = "\\end{document}";
pub(crate) const DISPLAYMATH_BEGIN: &str = "\\begin{displaymath}";
pub(crate) const DISPLAYMATH_END: &str = "\\end{displaymath}";
pub(crate) const EQUATION_BEGIN: &str = "\\begin{equation}";
pub(crate) const EQUATION_END: &str = "\\end{equation}";
pub(crate) const ALIGN_BEGIN: &str = "\\begin{align}";
pub(crate) const ALIGN_END: &str = "\\end{align}";
// joining '\begin' resp. '\end' with the environment name using format strings is not possible in
// every context where these constants are needed

// segments
/// Levels of segments in a LaTeX document.
/// A Node has the level of the next expected segment in the subtree beneath it including itself.
pub(crate) const SEGMENT_LEVELS: [(i8, &str); 7] = [
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
pub(crate) const SUBPARAGRAPH: &str = "subparagraph";

// math syntax
pub(crate) const DOUBLE_DOLLARS: &str = "$$";
pub(crate) const SQUARE_BRACKETS_LEFT: &str = "\\[";
pub(crate) const SQUARE_BRACKETS_RIGHT: &str = "\\]";

// prefixes and other delimiters
pub(crate) const KEYWORD_PREFIX: &str = "\\";
pub(crate) const COMMENT_PREFIX: &str = "%";
pub(crate) const BLOCK_BEGIN: &str = "{";
pub(crate) const BLOCK_END: &str = "}";
pub(crate) const BLOCK_DELIMITERS: (&str, &str) = (BLOCK_BEGIN, BLOCK_END);
pub(crate) const OPTIONS_BEGIN: &str = "[";
pub(crate) const OPTIONS_END: &str = "]";
