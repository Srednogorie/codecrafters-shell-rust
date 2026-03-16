use crate::enums::SpecialTokens;

pub struct PipelineStage {
    pub command: String,
    pub args: Vec<String>,
    pub redirect: Option<Redirect>, // stdout/stderr redirect for this stage
}

pub struct Redirect {
    pub token: SpecialTokens,
    pub target: String, // the file path, owned
}

pub struct History {
    entries: Vec<String>,
}
impl History {
    pub fn new() -> Self {
        Self { entries: Vec::new() }
    }
    pub fn add_entry(&mut self, entry: String) {
        self.entries.push(entry);
    }
    pub fn get_iter(&self) -> impl Iterator<Item = &str> + '_ {
        self.entries.iter().map(|e| e.as_str())
    }
}
