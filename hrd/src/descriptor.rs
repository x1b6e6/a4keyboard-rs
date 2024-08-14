use crate::Collection;
use crate::Report;

#[derive(Debug, PartialEq, Eq)]
pub struct Descriptor {
    pub main_collection: Collection,
}

impl Descriptor {
    pub(crate) fn new(main_collection: Collection) -> Self {
        Self { main_collection }
    }

    pub fn iter(&self) -> impl Iterator<Item = &Report> {
        self.main_collection.iter()
    }
}
