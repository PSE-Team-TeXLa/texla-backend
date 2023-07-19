pub struct StringificationOptions {
    pub include_comments: bool,
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
