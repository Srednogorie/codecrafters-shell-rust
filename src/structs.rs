use crate::enums::SpecialTokens;

pub struct PipelineStage {
    pub command: String,
    pub run_in_background: bool,
    pub args: Vec<String>,
    pub redirect: Option<Redirect>, // stdout/stderr redirect for this stage
}

pub struct Redirect {
    pub token: SpecialTokens,
    pub target: String, // the file path, owned
}

pub struct BackgroundJob {
    pub child: std::process::Child,
}
