//! `texla_constants` defines some tokens used by TEXLA to save special attributes in LaTeX Files.
pub(crate) const TEXLA: &str = "TEXLA";
pub(crate) const TEXLA_COMMENT_PREFIX: &str = "% TEXLA";
// joining '% ' with TEXLA using format strings is not possible in every context where these
// constants are needed (one could use the crate const_format here)
pub const TEXLA_COMMENT_DELIMITER_LEFT: char = '{';
pub const TEXLA_COMMENT_DELIMITER_RIGHT: char = '}';

pub const FILE_BEGIN_MARK: &str = "% TEXLA FILE BEGIN ";
pub const FILE_END_MARK: &str = "% TEXLA FILE END ";
// 'pub' instead of 'pub(crate)' used since it's needed in the other crate

pub(crate) const METADATA_MARK: &str = "% TEXLA METADATA ";
pub(crate) const METADATA_DELIMITER_LEFT: char = '(';
pub(crate) const METADATA_DELIMITER_RIGHT: char = ')';
pub(crate) const METADATA_SEPARATOR_KEY_VALUE: char = ':';
pub(crate) const METADATA_SEPARATOR_VALUES: char = ',';

pub(crate) const SKIPPED_CONTENT_MARK: &str = "…";
