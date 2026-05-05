use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Mode {
    Workspace,
    Preview,
    Srs,
    Graph,
    Files,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub(crate) struct TypstFile {
    pub(crate) path: String,
    pub(crate) deck: String,
    pub(crate) source: String,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct TypstPackageFile {
    pub(crate) namespace: String,
    pub(crate) name: String,
    pub(crate) version: String,
    pub(crate) path: String,
    pub(crate) bytes: Vec<u8>,
    pub(crate) source: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub(crate) struct Flashcard {
    pub(crate) id: String,
    pub(crate) deck: String,
    pub(crate) source_path: String,
    pub(crate) question: String,
    pub(crate) answer: String,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub(crate) struct ReviewState {
    pub(crate) id: String,
    pub(crate) deck: String,
    pub(crate) ease_factor: f64,
    pub(crate) interval: u32,
    pub(crate) repetitions: u32,
    pub(crate) next_review: i64,
    pub(crate) last_review: Option<i64>,
    pub(crate) last_quality: Option<u8>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub(crate) struct SrsProgress {
    pub(crate) cards: BTreeMap<String, ReviewState>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct GraphNode {
    pub(crate) id: String,
    pub(crate) label: String,
    pub(crate) directory: String,
    pub(crate) inbound: usize,
    pub(crate) outbound: usize,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct GraphEdge {
    pub(crate) source: String,
    pub(crate) target: String,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct Project {
    pub(crate) source_label: String,
    pub(crate) files: Vec<TypstFile>,
    pub(crate) package_files: Vec<TypstPackageFile>,
    pub(crate) cards: Vec<Flashcard>,
    pub(crate) graph_nodes: Vec<GraphNode>,
    pub(crate) graph_edges: Vec<GraphEdge>,
    pub(crate) progress: SrsProgress,
    pub(crate) active_file: Option<String>,
    pub(crate) message: String,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub(crate) struct SavedGitHubRepo {
    pub(crate) repo: String,
    pub(crate) git_ref: String,
    pub(crate) token: String,
}
