use std::collections::LinkedList;

use parking_lot::RwLock;
use unicode_segmentation::UnicodeSegmentation as _;

/// A global cache for all source code in the program.
pub static SOURCES: Sources = Sources::new();

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct SourceId(pub u32);

impl SourceId {
    pub fn get(&self) -> &'static Source {
        // TODO: remove expect and return an Option

        SOURCES
            .get(self.clone())
            .expect("Source ID should exist, ideally.")
    }
}

pub struct Sources {
    sources: RwLock<LinkedList<Source>>,
    paths: RwLock<Vec<String>>,
}

impl Sources {
    pub const fn new() -> Self {
        Self {
            sources: RwLock::new(LinkedList::new()),
            paths: RwLock::new(vec![]),
        }
    }

    pub fn add(&self, source: Source) -> SourceId {
        let name = source.name.clone();

        self.sources.write().push_back(source);
        self.paths.write().push(name);

        SourceId((self.sources.read().len() - 1) as u32)
    }

    pub fn get(&'static self, source_id: SourceId) -> Option<&'static Source> {
        unsafe {
            std::mem::transmute::<Option<&Source>, Option<&'static Source>>(
                self.sources.read().iter().nth(source_id.0 as usize),
            )
        }
    }

    pub fn get_by_path(&'static self, path: String) -> Option<&'static Source> {
        let index = self.paths.read().iter().position(|v| v == &path)? as u32;

        self.get(SourceId(index))
    }
}

pub struct Source {
    pub name: String,
    pub body: String,
    pub graphemes: Vec<String>,
}

impl Source {
    pub fn new(name: impl ToString, body: impl ToString) -> Self {
        let new_body = body.to_string();
        let graphemes = new_body.graphemes(true).map(|g| g.to_string()).collect();

        Self {
            name: name.to_string(),
            body: new_body,
            graphemes,
        }
    }

    pub fn from_path(path: impl AsRef<std::path::Path>) -> Self {
        // TODO: error handling

        let content = std::fs::read_to_string(path.as_ref());

        Self::new(
            path.as_ref().to_str().unwrap(),
            content.expect(&format!("file {:?} to exist", path.as_ref())),
        )
    }

    pub fn register(self) -> SourceId {
        SOURCES.add(self)
    }
}

impl std::fmt::Debug for Source {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("<Source name={:?}>", self.name))
    }
}
