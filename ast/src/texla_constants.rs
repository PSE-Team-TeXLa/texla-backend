// TODO: shouldn't these start with a newline?
pub(crate) const TEXLA: &str = "TEXLA";
pub(crate) const TEXLA_COMMENT_PREFIX: &str = "% TEXLA";
// joining '% ' with TEXLA using format strings is not possible in every context where these
// constants are needed

pub const FILE_BEGIN_MARK: &str = "% TEXLA FILE BEGIN ";
pub const FILE_END_MARK: &str = "% TEXLA FILE END ";
// 'pub' instead of 'pub(crate)' used since it's needed in the other crate

pub(crate) const METADATA_MARK: &str = "% TEXLA METADATA ";
pub(crate) const METADATA_DELIMITER_LEFT: &str = "(";
pub(crate) const METADATA_DELIMITER_RIGHT: &str = ")";
pub(crate) const METADATA_SEPARATOR_KEY_VALUE: &str = ":";
pub(crate) const METADATA_SEPARATOR_VALUES: &str = ",";

pub(crate) const SKIPPED_CONTENT_MARK: &str = "...";
