use crate::JournalName;

#[derive(Debug, Clone, derive_more::Display)]
#[display("[[{name}]]")]
pub struct Link {
    pub name: String,
}

pub trait ToLink {
    fn to_link(&self) -> Link;
}
impl<T: JournalName> ToLink for T {
    fn to_link(&self) -> Link {
        Link {
            name: self.to_journal_name(),
        }
    }
}

#[derive(Debug, Clone, derive_more::Display)]
#[display("{{{{embed {link}}}}}")]
pub struct Embedded {
    pub link: Link,
}

pub trait ToEmbedded {
    fn into_embedded(self) -> Embedded;
}
impl ToEmbedded for Link {
    fn into_embedded(self) -> Embedded {
        Embedded { link: self }
    }
}
