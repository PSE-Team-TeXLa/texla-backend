use serde::Deserialize;

/// StringificationOptions is used to specify how a given [super::Ast] should be converted to raw LaTeX Code.
#[derive(Deserialize, Debug)]
pub struct StringificationOptions {
    /// Whether or not to include regular LaTeX comments in the output.
    pub include_comments: bool,
    /// Whether or not to include comments used by TeXLa internally to save metadata about Elements in the input.
    pub include_metadata: bool,
}

impl Default for StringificationOptions {
    fn default() -> Self {
        Self {
            include_comments: true,
            include_metadata: true,
        }
    }
}
