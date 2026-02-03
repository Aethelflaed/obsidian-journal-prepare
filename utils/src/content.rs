#[derive(Debug, Clone, derive_more::Display, Eq, PartialEq)]
#[display("```{kind}\n{code}```")]
pub struct CodeBlock {
    kind: Kind,
    code: String,
}

#[derive(Debug, Clone, Eq, PartialEq, derive_more::IsVariant, derive_more::Display)]
enum Kind {
    #[display("toml")]
    Toml,
    #[display("{_0}")]
    Other(String),
}

impl Kind {
    pub const fn as_str(&self) -> &str {
        match self {
            Self::Toml => "toml",
            Self::Other(string) => string.as_str(),
        }
    }
}

impl From<&str> for Kind {
    fn from(string: &str) -> Self {
        match string {
            "toml" => Self::Toml,
            _ => Self::Other(string.to_owned()),
        }
    }
}

impl CodeBlock {
    #[must_use]
    pub fn new<S: Into<String>>(kind: &str, code: S) -> Self {
        Self {
            kind: kind.into(),
            code: code.into(),
        }
    }

    #[must_use]
    pub fn toml<S: Into<String>>(code: S) -> Self {
        Self {
            kind: Kind::Toml,
            code: code.into(),
        }
    }

    #[must_use]
    pub const fn kind(&self) -> &str {
        self.kind.as_str()
    }

    #[must_use]
    pub const fn code(&self) -> &str {
        self.code.as_str()
    }

    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.code.is_empty()
    }

    #[must_use]
    pub const fn is_toml(&self) -> bool {
        self.kind.is_toml()
    }
}
