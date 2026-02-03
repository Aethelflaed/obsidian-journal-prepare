#[derive(Debug, Clone, derive_more::Display, Eq, PartialEq)]
#[display("```{kind}\n{code}```")]
pub struct CodeBlock {
    pub kind: String,
    pub code: String,
}

impl CodeBlock {
    pub const fn is_empty(&self) -> bool {
        self.code.is_empty()
    }
}
