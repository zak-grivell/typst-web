use dioxus::prelude::*;
use regex::Regex;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet, HashMap};
#[cfg(not(target_arch = "wasm32"))]
use std::path::PathBuf;
use std::path::{Component, Path};

#[cfg(target_arch = "wasm32")]
const WEB_FONTS: &[&[u8]] = &[
    include_bytes!("../assets/fonts/LibertinusSerif-Regular.otf"),
    include_bytes!("../assets/fonts/LibertinusSerif-Bold.otf"),
    include_bytes!("../assets/fonts/LibertinusSerif-Italic.otf"),
    include_bytes!("../assets/fonts/NewCMMath-Regular.otf"),
];

const CSS: &str = r#"
:root {
  color-scheme: dark;
  --base: #303446;
  --mantle: #292c3c;
  --crust: #232634;
  --text: #c6d0f5;
  --muted: #a5adce;
  --surface0: #414559;
  --surface1: #51576d;
  --blue: #8caaee;
  --sky: #99d1db;
  --green: #a6d189;
  --yellow: #e5c890;
  --peach: #ef9f76;
  --red: #e78284;
  --mauve: #ca9ee6;
}
* { box-sizing: border-box; }
html, body, #main { height: 100%; }
body {
  margin: 0;
  min-height: 100vh;
  background: linear-gradient(180deg, var(--base), var(--crust));
  color: var(--text);
  font-family: Inter, ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
  overflow: hidden;
  -webkit-text-size-adjust: 100%;
}
button, input, textarea, select { font: inherit; }
button {
  border: 1px solid rgba(198, 208, 245, 0.12);
  background: rgba(65, 69, 89, 0.82);
  color: var(--text);
  min-height: 38px;
  border-radius: 8px;
  padding: 8px 12px;
  cursor: pointer;
  touch-action: manipulation;
}
button:hover, button.active { border-color: var(--blue); color: white; }
input, textarea, select {
  width: 100%;
  border: 1px solid rgba(198, 208, 245, 0.12);
  background: rgba(35, 38, 52, 0.82);
  color: var(--text);
  border-radius: 8px;
  padding: 10px 12px;
}
textarea { min-height: 160px; resize: vertical; }
.app { height: 100vh; display: flex; flex-direction: column; overflow: hidden; }
.toolbar {
  position: sticky;
  top: 0;
  z-index: 5;
  display: grid;
  grid-template-columns: minmax(220px, 1fr) minmax(240px, 1.2fr) auto;
  gap: 12px;
  align-items: center;
  padding: calc(10px + env(safe-area-inset-top)) 14px 10px;
  background: rgba(41, 44, 60, 0.92);
  border-bottom: 1px solid rgba(198, 208, 245, 0.12);
  backdrop-filter: blur(18px);
}
.brand { display: flex; align-items: center; gap: 10px; min-width: 0; }
.mark { width: 18px; height: 18px; border-radius: 50%; background: linear-gradient(135deg, var(--blue), var(--mauve)); box-shadow: 0 0 18px rgba(140,170,238,.45); }
.title { font-weight: 800; white-space: nowrap; }
.subtitle { color: var(--muted); font-size: 12px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.tabs { display: flex; gap: 8px; flex-wrap: wrap; justify-content: flex-end; }
.content { flex: 1; min-height: 0; display: grid; grid-template-columns: 320px minmax(0, 1fr); overflow: hidden; }
.sidebar { overflow: auto; border-right: 1px solid rgba(198, 208, 245, 0.1); background: rgba(35, 38, 52, 0.28); padding: 14px; }
.main { overflow: auto; padding: 16px; }
.panel { background: rgba(65, 69, 89, 0.46); border: 1px solid rgba(198, 208, 245, 0.12); border-radius: 8px; padding: 14px; }
.stack { display: grid; gap: 12px; }
.file-browser { margin-top: 14px; }
.row { display: flex; gap: 8px; align-items: center; flex-wrap: wrap; }
.file-row { width: 100%; text-align: left; display: grid; gap: 3px; margin-bottom: 6px; }
.file-name { font-weight: 700; overflow-wrap: anywhere; }
.muted { color: var(--muted); font-size: 13px; }
.pill { display: inline-flex; min-height: 28px; align-items: center; border: 1px solid rgba(198,208,245,.12); border-radius: 999px; padding: 4px 9px; color: var(--muted); background: rgba(65,69,89,.45); font-size: 12px; }
.svg-page { background: white; color: black; overflow: auto; border-radius: 8px; padding: 16px; -webkit-overflow-scrolling: touch; }
.svg-page svg { max-width: 100%; height: auto; display: block; margin: 0 auto; }
.error { border-color: rgba(231,130,132,.45); color: #ffd4d5; }
.source { white-space: pre-wrap; overflow-wrap: anywhere; font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace; font-size: 13px; }
.card-render { background: rgba(35,38,52,.38); border: 1px solid rgba(198,208,245,.1); border-radius: 8px; padding: 10px; overflow: auto; -webkit-overflow-scrolling: touch; }
.card-render .svg-page { padding: 0; background: transparent; }
.card-render svg { max-width: 100%; height: auto; display: block; }
.ratings { display: grid; grid-template-columns: repeat(4, minmax(0, 1fr)); gap: 10px; }
.again { background: var(--red); color: var(--crust); font-weight: 800; }
.hard { background: var(--peach); color: var(--crust); font-weight: 800; }
.good { background: var(--green); color: var(--crust); font-weight: 800; }
.easy { background: var(--sky); color: var(--crust); font-weight: 800; }
.primary { background: var(--blue); color: var(--crust); font-weight: 800; }
.track { height: 8px; border-radius: 999px; background: rgba(35,38,52,.85); overflow: hidden; }
.bar { height: 100%; background: linear-gradient(90deg, var(--blue), var(--green)); }
.graph { width: 100%; min-height: 620px; background: radial-gradient(circle at 12% 12%, rgba(140,170,238,.18), transparent 24%), linear-gradient(180deg, rgba(35,38,52,.96), rgba(30,32,48,.96)); }
.node-list { display: grid; grid-template-columns: repeat(auto-fill, minmax(230px, 1fr)); gap: 10px; }
@media (max-width: 900px) {
  body { overflow: auto; }
  .app { height: auto; min-height: 100dvh; overflow: visible; }
  .toolbar {
    grid-template-columns: 1fr;
    gap: 8px;
    padding: calc(8px + env(safe-area-inset-top)) 10px 8px;
  }
  .brand { gap: 8px; }
  .mark { width: 16px; height: 16px; flex: 0 0 auto; }
  .subtitle { max-width: 100%; }
  .tabs { display: grid; grid-template-columns: repeat(4, minmax(0, 1fr)); gap: 6px; justify-content: stretch; }
  .tabs button { min-width: 0; min-height: 42px; padding: 7px 6px; }
  .content { display: block; overflow: visible; }
  .sidebar {
    max-height: none;
    overflow: visible;
    border-right: 0;
    border-bottom: 1px solid rgba(198,208,245,.1);
    padding: 10px;
  }
  .main { overflow: visible; padding: 10px; }
  .panel { padding: 11px; }
  .source-panel { gap: 8px; }
  .source-panel input, .source-panel button { min-height: 42px; }
  .file-browser {
    max-height: 30dvh;
    overflow: auto;
    margin-top: 10px;
    padding-right: 2px;
    -webkit-overflow-scrolling: touch;
  }
  .file-row { min-height: 48px; margin-bottom: 5px; padding: 8px 10px; }
  .file-name { font-size: 14px; }
  .muted { font-size: 12px; }
  .pill { max-width: 100%; overflow-wrap: anywhere; }
  .svg-page { max-height: 68dvh; padding: 10px; }
  .source { font-size: 12px; line-height: 1.45; }
  .card-render { padding: 8px; }
  .graph { min-height: 420px; }
  .node-list { grid-template-columns: 1fr; }
  .ratings { grid-template-columns: 1fr 1fr; }
}
@media (max-width: 520px) {
  button { min-height: 42px; padding: 8px 10px; }
  input, textarea, select { min-height: 42px; padding: 9px 10px; }
  .title { font-size: 15px; }
  .tabs button { font-size: 13px; }
  .row { gap: 6px; }
  .ratings { gap: 8px; }
}
"#;

thread_local! {
    static RENDER_CACHE: RefCell<BTreeMap<String, Result<String, String>>> = RefCell::new(BTreeMap::new());
    #[cfg(target_arch = "wasm32")]
    static WEB_FONT_CACHE: RefCell<Option<Vec<typst::text::Font>>> = const { RefCell::new(None) };
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Mode {
    Preview,
    Srs,
    Graph,
    Files,
}

impl Mode {
    fn label(self) -> &'static str {
        match self {
            Mode::Preview => "Preview",
            Mode::Srs => "SRS",
            Mode::Graph => "Graph",
            Mode::Files => "Files",
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
struct TypstFile {
    path: String,
    deck: String,
    source: String,
}

#[derive(Clone, Debug, Default, PartialEq)]
struct TypstPackageFile {
    namespace: String,
    name: String,
    version: String,
    path: String,
    bytes: Vec<u8>,
    source: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
struct Flashcard {
    id: String,
    deck: String,
    source_path: String,
    question: String,
    answer: String,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
struct ReviewState {
    id: String,
    deck: String,
    ease_factor: f64,
    interval: u32,
    repetitions: u32,
    next_review: i64,
    last_review: Option<i64>,
    last_quality: Option<u8>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
struct SrsProgress {
    cards: BTreeMap<String, ReviewState>,
}

#[derive(Clone, Debug, Default, PartialEq)]
struct GraphNode {
    id: String,
    label: String,
    directory: String,
    inbound: usize,
    outbound: usize,
}

#[derive(Clone, Debug, Default, PartialEq)]
struct GraphEdge {
    source: String,
    target: String,
}

#[derive(Clone, Debug, Default, PartialEq)]
struct Project {
    source_label: String,
    files: Vec<TypstFile>,
    package_files: Vec<TypstPackageFile>,
    cards: Vec<Flashcard>,
    graph_nodes: Vec<GraphNode>,
    graph_edges: Vec<GraphEdge>,
    progress: SrsProgress,
    active_file: Option<String>,
    message: String,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
struct SavedGitHubRepo {
    repo: String,
    git_ref: String,
    token: String,
}

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let mut mode = use_signal(|| Mode::Files);
    let project = use_signal(Project::sample);
    let mut active_file = use_signal(|| project.read().active_file.clone());
    let mut query = use_signal(String::new);
    let local_path = use_signal(String::new);
    let github_repo = use_signal(String::new);
    let github_ref = use_signal(|| "main".to_string());
    let github_token = use_signal(String::new);
    let srs = use_signal(SrsSession::default);

    let project_value = project.read().clone();
    let source_label = project_value.source_label.clone();
    let active = active_file
        .read()
        .clone()
        .unwrap_or_else(|| "no file selected".to_string());
    let filtered = filter_files(&project_value.files, &query.read());

    rsx! {
        style { "{CSS}" }
        div { class: "app",
            header { class: "toolbar",
                div { class: "brand",
                    div { class: "mark" }
                    div {
                        div { class: "title", "typst-web" }
                        div { class: "subtitle", "{mode.read().label()} · {source_label} · {active}" }
                    }
                }
                input {
                    value: "{query}",
                    placeholder: "Search files...",
                    oninput: move |event| query.set(event.value())
                }
                div { class: "tabs",
                    for tab in [Mode::Preview, Mode::Srs, Mode::Graph, Mode::Files] {
                        button {
                            class: if *mode.read() == tab { "active" } else { "" },
                            onclick: move |_| mode.set(tab),
                            "{tab.label()}"
                        }
                    }
                }
            }
            div { class: "content",
                aside { class: "sidebar",
                    SourcePanel {
                        local_path,
                        github_repo,
                        github_ref,
                        github_token,
                        project,
                        active_file
                    }
                    div { class: "file-browser stack",
                        div { class: "row",
                            span { class: "pill", "{project_value.files.len()} files" }
                            span { class: "pill", "{project_value.cards.len()} cards" }
                            span { class: "pill", "{project_value.graph_edges.len()} links" }
                        }
                        for file in filtered {
                            button {
                                class: "file-row",
                                onclick: {
                                    let path = file.path.clone();
                                    move |_| {
                                        active_file.set(Some(path.clone()));
                                        mode.set(Mode::Preview);
                                    }
                                },
                                div { class: "file-name", "{file_name(&file.path)}" }
                                div { class: "muted", "{file.path}" }
                            }
                        }
                    }
                }
                main { class: "main",
                    match *mode.read() {
                        Mode::Preview => rsx! { PreviewView { project: project_value.clone(), active_file: active_file.read().clone() } },
                        Mode::Srs => rsx! { SrsView { project, srs } },
                        Mode::Graph => rsx! { GraphView { project: project_value.clone() } },
                        Mode::Files => rsx! { FilesView { project: project_value.clone() } },
                    }
                }
            }
        }
    }
}

#[component]
fn SourcePanel(
    mut local_path: Signal<String>,
    mut github_repo: Signal<String>,
    mut github_ref: Signal<String>,
    mut github_token: Signal<String>,
    mut project: Signal<Project>,
    mut active_file: Signal<Option<String>>,
) -> Element {
    let mut saved_github_repos = use_signal(load_saved_github_repos);
    let saved = saved_github_repos.read().clone();

    rsx! {
        div { class: "panel stack source-panel",
            div { class: "file-name", "Workspace" }
            input {
                value: "{local_path}",
                placeholder: "Local project path",
                oninput: move |event| local_path.set(event.value())
            }
            button {
                onclick: move |_| {
                    start_local_load(local_path.read().to_string(), project, active_file);
                },
                "Load Local"
            }
            if !saved.is_empty() {
                select {
                    value: "",
                    onchange: move |event| {
                        if let Ok(index) = event.value().parse::<usize>() {
                            if let Some(entry) = saved_github_repos.read().get(index).cloned() {
                                github_repo.set(entry.repo);
                                github_ref.set(entry.git_ref);
                                github_token.set(entry.token);
                            }
                        }
                    },
                    option { value: "", "Saved GitHub repos" }
                    for (index, entry) in saved.iter().enumerate() {
                        option { value: "{index}", "{entry.repo} @ {entry.git_ref}" }
                    }
                }
            }
            input {
                value: "{github_repo}",
                placeholder: "GitHub owner/repo",
                oninput: move |event| github_repo.set(event.value())
            }
            input {
                value: "{github_ref}",
                placeholder: "branch or ref",
                oninput: move |event| github_ref.set(event.value())
            }
            input {
                r#type: "password",
                value: "{github_token}",
                placeholder: "token for private repo",
                oninput: move |event| github_token.set(event.value())
            }
            button {
                onclick: move |_| {
                    let repo = github_repo.read().to_string();
                    let git_ref = github_ref.read().to_string();
                    let token = github_token.read().to_string();
                    if let Some(next) = remember_github_repo(repo.clone(), git_ref.clone(), token.clone()) {
                        saved_github_repos.set(next);
                    }
                    start_github_load(
                        repo,
                        git_ref,
                        token,
                        project,
                        active_file,
                    );
                },
                "Load GitHub"
            }
            div { class: "muted", "{project.read().message}" }
        }
    }
}

#[cfg(target_arch = "wasm32")]
const SAVED_GITHUB_REPOS_KEY: &str = "typst-web:saved-github-repos";

fn load_saved_github_repos() -> Vec<SavedGitHubRepo> {
    load_saved_github_repos_impl().unwrap_or_default()
}

#[cfg(target_arch = "wasm32")]
fn load_saved_github_repos_impl() -> Option<Vec<SavedGitHubRepo>> {
    let raw = browser_storage()?.get_item(SAVED_GITHUB_REPOS_KEY).ok()??;
    serde_json::from_str(&raw).ok()
}

#[cfg(not(target_arch = "wasm32"))]
fn load_saved_github_repos_impl() -> Option<Vec<SavedGitHubRepo>> {
    None
}

fn remember_github_repo(
    repo: String,
    git_ref: String,
    token: String,
) -> Option<Vec<SavedGitHubRepo>> {
    let repo = repo.trim().to_string();
    let git_ref = git_ref.trim().to_string();
    if repo.is_empty() {
        return None;
    }

    let mut saved = load_saved_github_repos();
    saved.retain(|entry| entry.repo != repo || entry.git_ref != git_ref);
    saved.insert(
        0,
        SavedGitHubRepo {
            repo,
            git_ref,
            token,
        },
    );
    saved.truncate(12);
    persist_saved_github_repos(&saved);
    Some(saved)
}

#[cfg(target_arch = "wasm32")]
fn persist_saved_github_repos(saved: &[SavedGitHubRepo]) {
    let Some(storage) = browser_storage() else {
        return;
    };
    if let Ok(raw) = serde_json::to_string(saved) {
        let _ = storage.set_item(SAVED_GITHUB_REPOS_KEY, &raw);
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn persist_saved_github_repos(_saved: &[SavedGitHubRepo]) {}

#[cfg(target_arch = "wasm32")]
fn browser_storage() -> Option<web_sys::Storage> {
    web_sys::window()?.local_storage().ok().flatten()
}

#[component]
fn PreviewView(project: Project, active_file: Option<String>) -> Element {
    let active = active_file
        .as_ref()
        .and_then(|path| project.files.iter().find(|file| &file.path == path))
        .or_else(|| project.files.first());

    match active {
        Some(file) => {
            let rendered = cached_render_typst_svg(&project, file);
            rsx! {
                div { class: "stack",
                    div { class: "row",
                        span { class: "pill", "{file.path}" }
                        span { class: "pill", "{file.deck}" }
                    }
                    match rendered {
                        Ok(svg) => rsx! {
                            div { class: "svg-page", dangerous_inner_html: "{svg}" }
                        },
                        Err(err) => rsx! {
                            div { class: "panel error stack",
                                div { class: "file-name", "Render diagnostic" }
                                div { class: "source", "{err}" }
                            }
                            div { class: "panel source", "{file.source}" }
                        },
                    }
                }
            }
        }
        None => {
            rsx! { div { class: "panel", "Load a local directory or GitHub repository containing .typ files." } }
        }
    }
}

#[derive(Clone, Default)]
struct SrsSession {
    queue: Vec<String>,
    index: usize,
    showing_answer: bool,
    mode: ReviewMode,
    deck: Option<String>,
}

#[derive(Clone, Copy, Default, Eq, PartialEq)]
enum ReviewMode {
    #[default]
    Due,
    All,
    Cram,
    Redo,
}

impl ReviewMode {
    fn label(self) -> &'static str {
        match self {
            ReviewMode::Due => "Due",
            ReviewMode::All => "All",
            ReviewMode::Cram => "Cram",
            ReviewMode::Redo => "Redo",
        }
    }
}

#[component]
fn SrsView(mut project: Signal<Project>, mut srs: Signal<SrsSession>) -> Element {
    let current_project = project.read().clone();
    let current_session = srs.read().clone();
    let decks = card_decks(&current_project.cards);
    let total = current_session.queue.len();
    let done = current_session.index.min(total);
    let card = current_session
        .queue
        .get(current_session.index)
        .and_then(|id| current_project.cards.iter().find(|card| &card.id == id))
        .cloned();
    let pct = if total == 0 { 0 } else { done * 100 / total };

    rsx! {
        div { class: "stack",
            div { class: "panel stack",
                div { class: "row",
                    for review_mode in [ReviewMode::Due, ReviewMode::All, ReviewMode::Cram, ReviewMode::Redo] {
                        button {
                            class: if current_session.mode == review_mode { "active" } else { "" },
                            onclick: move |event| {
                                event.stop_propagation();
                                let mut next = srs.read().clone();
                                next.mode = review_mode;
                                next.queue = build_review_queue(&project.read(), review_mode, next.deck.as_deref());
                                next.index = 0;
                                next.showing_answer = false;
                                srs.set(next);
                            },
                            "{review_mode.label()}"
                        }
                    }
                    button {
                        onclick: move |event| {
                            event.stop_propagation();
                            let mode_value = srs.read().mode;
                            let mut next = srs.read().clone();
                            next.queue = build_review_queue(&project.read(), mode_value, next.deck.as_deref());
                            next.index = 0;
                            next.showing_answer = false;
                            srs.set(next);
                        },
                        "Start"
                    }
                    span { class: "pill", "{done}/{total}" }
                }
                div { class: "row",
                    button {
                        class: if current_session.deck.is_none() { "active" } else { "" },
                        onclick: move |event| {
                            event.stop_propagation();
                            let mut next = srs.read().clone();
                            next.deck = None;
                            next.queue = build_review_queue(&project.read(), next.mode, None);
                            next.index = 0;
                            next.showing_answer = false;
                            srs.set(next);
                        },
                        "All decks"
                    }
                    for deck in decks {
                        button {
                            class: if current_session.deck.as_deref() == Some(deck.as_str()) { "active" } else { "" },
                            onclick: {
                                let deck = deck.clone();
                                move |event| {
                                    event.stop_propagation();
                                    let mut next = srs.read().clone();
                                    next.deck = Some(deck.clone());
                                    next.queue = build_review_queue(&project.read(), next.mode, next.deck.as_deref());
                                    next.index = 0;
                                    next.showing_answer = false;
                                    srs.set(next);
                                }
                            },
                            "{deck}"
                        }
                    }
                }
                div { class: "track", div { class: "bar", style: "width: {pct}%;" } }
            }
            match card {
                Some(card) => rsx! {
                    div {
                        class: "panel stack",
                        onclick: move |event| {
                            event.stop_propagation();
                            let mut next = srs.read().clone();
                            next.showing_answer = true;
                            srs.set(next);
                        },
                        div { class: "row",
                            span { class: "pill", "{card.deck}" }
                            span { class: "pill", "{card.source_path}" }
                        }
                        div { class: "file-name", "Question" }
                        TypstCardContent { project: current_project.clone(), card: card.clone(), side: CardSide::Question }
                        if current_session.showing_answer {
                            div { class: "file-name", "Answer" }
                            TypstCardContent { project: current_project.clone(), card: card.clone(), side: CardSide::Answer }
                        }
                    }
                    if current_session.showing_answer {
                        div { class: "ratings",
                            RatingButton { label: "Again", class_name: "again", quality: 1, card_id: card.id.clone(), project, srs }
                            RatingButton { label: "Hard", class_name: "hard", quality: 3, card_id: card.id.clone(), project, srs }
                            RatingButton { label: "Good", class_name: "good", quality: 4, card_id: card.id.clone(), project, srs }
                            RatingButton { label: "Easy", class_name: "easy", quality: 5, card_id: card.id.clone(), project, srs }
                        }
                    } else {
                        button {
                            class: "primary",
                            onclick: move |event| {
                                event.stop_propagation();
                                let mut next = srs.read().clone();
                                next.showing_answer = true;
                                srs.set(next);
                            },
                            "Show Answer"
                        }
                    }
                },
                None => rsx! {
                    div { class: "panel stack",
                        div { class: "file-name", "No card queued" }
                        div { class: "muted", "Choose a review mode and press Start. Due mode may be empty when nothing is scheduled." }
                    }
                }
            }
        }
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum CardSide {
    Question,
    Answer,
}

impl CardSide {
    fn label(self) -> &'static str {
        match self {
            CardSide::Question => "question",
            CardSide::Answer => "answer",
        }
    }

    fn source(self, card: &Flashcard) -> &str {
        match self {
            CardSide::Question => &card.question,
            CardSide::Answer => &card.answer,
        }
    }
}

#[component]
fn TypstCardContent(project: Project, card: Flashcard, side: CardSide) -> Element {
    let file = card_typst_file(&card, side);
    match cached_render_typst_svg(&project, &file) {
        Ok(svg) => rsx! {
            div { class: "card-render",
                div { class: "svg-page", dangerous_inner_html: "{svg}" }
            }
        },
        Err(err) => rsx! {
            div { class: "panel error stack",
                div { class: "file-name", "Render diagnostic" }
                div { class: "source", "{err}" }
                div { class: "source", "{side.source(&card)}" }
            }
        },
    }
}

#[component]
fn RatingButton(
    label: &'static str,
    class_name: &'static str,
    quality: u8,
    card_id: String,
    mut project: Signal<Project>,
    mut srs: Signal<SrsSession>,
) -> Element {
    rsx! {
        button {
            class: "{class_name}",
            onclick: move |event| {
                event.stop_propagation();
                let mut next_project = project.read().clone();
                if let Some(card) = next_project.cards.iter().find(|card| card.id == card_id).cloned() {
                    review_card(&mut next_project.progress, &card, quality);
                    persist_progress(&next_project);
                    project.set(next_project);
                }
                let mut next = srs.read().clone();
                next.index = next.index.saturating_add(1).min(next.queue.len());
                next.showing_answer = false;
                srs.set(next);
            },
            "{label}"
        }
    }
}

#[component]
fn GraphView(project: Project) -> Element {
    let width = 960.0;
    let height = 620.0;
    let positions = graph_positions(&project.graph_nodes, width, height);

    rsx! {
        div { class: "stack",
            div { class: "row",
                span { class: "pill", "{project.graph_nodes.len()} notes" }
                span { class: "pill", "{project.graph_edges.len()} links" }
            }
            svg { class: "graph", view_box: "0 0 {width} {height}",
                for edge in &project.graph_edges {
                    if let (Some(a), Some(b)) = (positions.get(&edge.source), positions.get(&edge.target)) {
                        line {
                            x1: "{a.0}", y1: "{a.1}", x2: "{b.0}", y2: "{b.1}",
                            stroke: "rgba(140,170,238,.35)", stroke_width: "1.4"
                        }
                    }
                }
                for node in &project.graph_nodes {
                    if let Some((x, y)) = positions.get(&node.id) {
                        circle {
                            cx: "{x}", cy: "{y}", r: "{12 + node.inbound + node.outbound}",
                            fill: "#babbf1", stroke: "#8caaee", stroke_width: "2"
                        }
                        text {
                            x: "{x + 14.0}", y: "{y + 4.0}", fill: "#c6d0f5", font_size: "12",
                            "{node.label}"
                        }
                    }
                }
            }
            div { class: "node-list",
                for node in &project.graph_nodes {
                    div { class: "panel",
                        div { class: "file-name", "{node.id}" }
                        div { class: "muted", "{node.directory} · in {node.inbound} · out {node.outbound}" }
                    }
                }
            }
        }
    }
}

#[component]
fn FilesView(project: Project) -> Element {
    let mut by_deck: BTreeMap<String, Vec<TypstFile>> = BTreeMap::new();
    for file in &project.files {
        by_deck
            .entry(file.deck.clone())
            .or_default()
            .push(file.clone());
    }

    rsx! {
        div { class: "stack",
            for (deck, files) in by_deck {
                div { class: "panel stack",
                    div { class: "row",
                        div { class: "file-name", "{deck}" }
                        span { class: "pill", "{files.len()} files" }
                    }
                    for file in files {
                        div {
                            div { class: "file-name", "{file.path}" }
                            div { class: "muted", "{card_count_for(&project.cards, &file.path)} cards" }
                            div { class: "source", "{excerpt(&file.source)}" }
                        }
                    }
                }
            }
        }
    }
}

impl Project {
    fn sample() -> Self {
        let files = vec![
            TypstFile {
                path: "index.typ".to_string(),
                deck: "root".to_string(),
                source: "#import \"setup.typ\": flashcard\n= Index\nSee \"./patterns/observer.typ\".\n#flashcard(\"What does Observer decouple?\", \"Subjects from subscribers.\")".to_string(),
            },
            TypstFile {
                path: "patterns/observer.typ".to_string(),
                deck: "patterns".to_string(),
                source: "= Observer\nLinks back to \"../index.typ\".\n#flashcard(\"When is Observer useful?\", \"When many views react to state changes.\")".to_string(),
            },
            TypstFile {
                path: "setup.typ".to_string(),
                deck: "root".to_string(),
                source: "#let flashcard(q, a) = []".to_string(),
            },
        ];
        build_project(
            "sample workspace".to_string(),
            files,
            "Ready. Load a local path or GitHub repo to replace the sample.".to_string(),
        )
    }
}

fn build_project(source_label: String, files: Vec<TypstFile>, message: String) -> Project {
    build_project_with_packages(source_label, files, Vec::new(), message)
}

fn build_project_with_packages(
    source_label: String,
    mut files: Vec<TypstFile>,
    mut package_files: Vec<TypstPackageFile>,
    message: String,
) -> Project {
    files.sort_by(|a, b| a.path.cmp(&b.path));
    package_files.sort_by(|a, b| {
        (&a.namespace, &a.name, &a.version, &a.path).cmp(&(
            &b.namespace,
            &b.name,
            &b.version,
            &b.path,
        ))
    });
    let cards = extract_cards(&files);
    let (graph_nodes, graph_edges) = build_graph(&files);
    let mut progress = load_progress_for(&source_label).unwrap_or_default();
    for card in &cards {
        progress
            .cards
            .entry(card.id.clone())
            .or_insert_with(|| ReviewState {
                id: card.id.clone(),
                deck: card.deck.clone(),
                ease_factor: 2.5,
                interval: 0,
                repetitions: 0,
                next_review: 0,
                last_review: None,
                last_quality: None,
            });
    }
    let active_file = files
        .iter()
        .find(|file| file.path != "setup.typ")
        .map(|file| file.path.clone());
    Project {
        source_label,
        files,
        package_files,
        cards,
        graph_nodes,
        graph_edges,
        progress,
        active_file,
        message,
    }
}

fn filter_files(files: &[TypstFile], query: &str) -> Vec<TypstFile> {
    let needle = query.trim().to_lowercase();
    files
        .iter()
        .filter(|file| needle.is_empty() || fuzzy_match(&file.path.to_lowercase(), &needle))
        .take(50)
        .cloned()
        .collect()
}

fn fuzzy_match(path: &str, needle: &str) -> bool {
    let mut chars = needle.chars();
    let Some(mut current) = chars.next() else {
        return true;
    };
    for ch in path.chars() {
        if ch == current {
            match chars.next() {
                Some(next) => current = next,
                None => return true,
            }
        }
    }
    false
}

fn extract_cards(files: &[TypstFile]) -> Vec<Flashcard> {
    let mut cards = Vec::new();
    for file in files {
        for (question, answer) in parse_flashcards(&file.source) {
            let mut hasher = Sha256::new();
            hasher.update(file.deck.as_bytes());
            hasher.update(file.path.as_bytes());
            hasher.update(question.as_bytes());
            let id = format!("{:x}", hasher.finalize());
            cards.push(Flashcard {
                id,
                deck: file.deck.clone(),
                source_path: file.path.clone(),
                question,
                answer,
            });
        }
    }
    cards
}

fn parse_flashcards(source: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let bytes = source.as_bytes();
    let mut index = 0;
    while let Some(pos) = source[index..].find("flashcard(") {
        let start = index + pos + "flashcard(".len();
        if let Some(end) = find_balanced_close(bytes, start - 1) {
            let inside = &source[start..end];
            if let Some((q, a)) = split_top_level_args(inside) {
                out.push((clean_typst_arg(q), clean_typst_arg(a)));
            }
            index = end + 1;
        } else {
            break;
        }
    }
    out
}

fn find_balanced_close(bytes: &[u8], open: usize) -> Option<usize> {
    let mut depth = 0i32;
    let mut quote = false;
    let mut escape = false;
    for (index, byte) in bytes.iter().enumerate().skip(open) {
        if quote {
            if escape {
                escape = false;
            } else if *byte == b'\\' {
                escape = true;
            } else if *byte == b'"' {
                quote = false;
            }
            continue;
        }
        match *byte {
            b'"' => quote = true,
            b'(' => depth += 1,
            b')' => {
                depth -= 1;
                if depth == 0 {
                    return Some(index);
                }
            }
            _ => {}
        }
    }
    None
}

fn split_top_level_args(input: &str) -> Option<(&str, &str)> {
    let mut depth = 0i32;
    let mut quote = false;
    let mut escape = false;
    for (index, ch) in input.char_indices() {
        if quote {
            if escape {
                escape = false;
            } else if ch == '\\' {
                escape = true;
            } else if ch == '"' {
                quote = false;
            }
            continue;
        }
        match ch {
            '"' => quote = true,
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' => depth -= 1,
            ',' if depth == 0 => return Some((&input[..index], &input[index + 1..])),
            _ => {}
        }
    }
    None
}

fn clean_typst_arg(input: &str) -> String {
    let trimmed = input.trim();
    if trimmed.len() >= 2 && trimmed.starts_with('"') && trimmed.ends_with('"') {
        trimmed[1..trimmed.len() - 1].replace("\\\"", "\"")
    } else if trimmed.len() >= 2 && trimmed.starts_with('[') && trimmed.ends_with(']') {
        trimmed[1..trimmed.len() - 1].trim().to_string()
    } else {
        trimmed.to_string()
    }
}

fn build_graph(files: &[TypstFile]) -> (Vec<GraphNode>, Vec<GraphEdge>) {
    let paths: BTreeSet<String> = files
        .iter()
        .filter(|file| file.path != "setup.typ")
        .map(|file| file.path.clone())
        .collect();
    let link_re = Regex::new(r#""([^"]+\.typ)""#).expect("valid regex");
    let mut edges = Vec::new();
    let mut inbound: HashMap<String, usize> = HashMap::new();
    let mut outbound: HashMap<String, usize> = HashMap::new();

    for file in files.iter().filter(|file| paths.contains(&file.path)) {
        for cap in link_re.captures_iter(&file.source) {
            if let Some(target) = normalize_link(&file.path, &cap[1]) {
                if paths.contains(&target) {
                    edges.push(GraphEdge {
                        source: file.path.clone(),
                        target: target.clone(),
                    });
                    *outbound.entry(file.path.clone()).or_default() += 1;
                    *inbound.entry(target).or_default() += 1;
                }
            }
        }
    }

    let nodes = paths
        .into_iter()
        .map(|path| GraphNode {
            label: file_name(&path),
            directory: deck_for_path(&path),
            inbound: inbound.get(&path).copied().unwrap_or(0),
            outbound: outbound.get(&path).copied().unwrap_or(0),
            id: path,
        })
        .collect();

    (nodes, edges)
}

fn normalize_link(source_path: &str, link: &str) -> Option<String> {
    if link.starts_with("http://") || link.starts_with("https://") {
        return None;
    }
    let base = Path::new(source_path)
        .parent()
        .unwrap_or_else(|| Path::new(""));
    normalize_relative_path(&base.join(link))
}

fn normalize_relative_path(path: &Path) -> Option<String> {
    let mut parts = Vec::new();
    for component in path.components() {
        match component {
            Component::Normal(part) => parts.push(part.to_string_lossy().to_string()),
            Component::CurDir => {}
            Component::ParentDir => {
                parts.pop()?;
            }
            _ => return None,
        }
    }
    Some(parts.join("/"))
}

fn build_review_queue(project: &Project, mode: ReviewMode, deck: Option<&str>) -> Vec<String> {
    let now = now_ms();
    project
        .cards
        .iter()
        .filter(|card| {
            if deck.is_some_and(|deck| card.deck != deck) {
                return false;
            }
            let state = project.progress.cards.get(&card.id);
            match mode {
                ReviewMode::Due => state.map(|s| s.next_review <= now).unwrap_or(true),
                ReviewMode::All | ReviewMode::Cram => true,
                ReviewMode::Redo => state
                    .and_then(|s| s.last_quality)
                    .map(|quality| quality < 5)
                    .unwrap_or(false),
            }
        })
        .map(|card| card.id.clone())
        .collect()
}

fn card_decks(cards: &[Flashcard]) -> Vec<String> {
    cards
        .iter()
        .map(|card| card.deck.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn review_card(progress: &mut SrsProgress, card: &Flashcard, quality: u8) {
    let now = now_ms();
    let state = progress
        .cards
        .entry(card.id.clone())
        .or_insert_with(|| ReviewState {
            id: card.id.clone(),
            deck: card.deck.clone(),
            ease_factor: 2.5,
            interval: 0,
            repetitions: 0,
            next_review: 0,
            last_review: None,
            last_quality: None,
        });

    if quality < 3 {
        state.repetitions = 0;
        state.interval = 1;
    } else {
        state.repetitions += 1;
        state.interval = match state.repetitions {
            1 => 1,
            2 => 6,
            _ => ((state.interval as f64) * state.ease_factor)
                .round()
                .max(1.0) as u32,
        };
    }

    let q = quality as f64;
    state.ease_factor =
        (state.ease_factor + (0.1 - (5.0 - q) * (0.08 + (5.0 - q) * 0.02))).max(1.3);
    state.last_review = Some(now);
    state.last_quality = Some(quality);
    state.next_review = now + (state.interval as i64 * 86_400_000);
}

#[cfg(not(target_arch = "wasm32"))]
fn now_ms() -> i64 {
    time::OffsetDateTime::now_utc().unix_timestamp() * 1000
}

#[cfg(target_arch = "wasm32")]
fn now_ms() -> i64 {
    js_sys::Date::now() as i64
}

fn graph_positions(nodes: &[GraphNode], width: f64, height: f64) -> HashMap<String, (f64, f64)> {
    let count = nodes.len().max(1) as f64;
    nodes
        .iter()
        .enumerate()
        .map(|(index, node)| {
            let angle = index as f64 / count * std::f64::consts::TAU;
            let radius = 120.0 + count.min(18.0) * 10.0;
            let x = width / 2.0 + angle.cos() * radius;
            let y = height / 2.0 + angle.sin() * radius;
            (node.id.clone(), (x, y))
        })
        .collect()
}

#[cfg(not(target_arch = "wasm32"))]
fn load_local_workspace(root: &str) -> Project {
    let root = PathBuf::from(root.trim());
    if root.as_os_str().is_empty() {
        return Project {
            message: "Enter a local project path.".to_string(),
            ..Project::sample()
        };
    }
    match scan_local_typst_files(&root) {
        Ok(files) if files.is_empty() => build_project(
            root.display().to_string(),
            files,
            "No .typ files found.".to_string(),
        ),
        Ok(files) => build_project(
            root.display().to_string(),
            files,
            "Loaded local workspace.".to_string(),
        ),
        Err(err) => Project {
            message: format!("Local load failed: {err}"),
            ..Project::sample()
        },
    }
}

#[cfg(target_arch = "wasm32")]
fn load_local_workspace(_root: &str) -> Project {
    Project { message: "Direct directory scanning is unavailable in the web sandbox. Use GitHub or an imported snapshot.".to_string(), ..Project::sample() }
}

#[cfg(not(target_arch = "wasm32"))]
fn start_local_load(
    root: String,
    mut project: Signal<Project>,
    mut active_file: Signal<Option<String>>,
) {
    spawn(async move {
        let loaded = match tokio::task::spawn_blocking(move || load_local_workspace(&root)).await {
            Ok(project) => project,
            Err(err) => Project {
                message: format!("Local load failed: background task failed: {err}"),
                ..Project::sample()
            },
        };
        active_file.set(loaded.active_file.clone());
        project.set(loaded);
    });
}

#[cfg(target_arch = "wasm32")]
fn start_local_load(
    root: String,
    mut project: Signal<Project>,
    mut active_file: Signal<Option<String>>,
) {
    let loaded = load_local_workspace(&root);
    active_file.set(loaded.active_file.clone());
    project.set(loaded);
}

#[cfg(not(target_arch = "wasm32"))]
fn scan_local_typst_files(root: &Path) -> Result<Vec<TypstFile>, String> {
    let mut files = Vec::new();
    scan_dir(root, root, &mut files)?;
    Ok(files)
}

#[cfg(not(target_arch = "wasm32"))]
fn scan_dir(root: &Path, dir: &Path, files: &mut Vec<TypstFile>) -> Result<(), String> {
    let entries = std::fs::read_dir(dir).map_err(|err| err.to_string())?;
    for entry in entries {
        let entry = entry.map_err(|err| err.to_string())?;
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') || name == "node_modules" {
            continue;
        }
        if path.is_dir() {
            scan_dir(root, &path, files)?;
        } else if path.extension().is_some_and(|ext| ext == "typ") {
            let rel = path.strip_prefix(root).map_err(|err| err.to_string())?;
            let rel =
                normalize_relative_path(rel).ok_or_else(|| "invalid relative path".to_string())?;
            let source = std::fs::read_to_string(&path).map_err(|err| err.to_string())?;
            files.push(TypstFile {
                deck: deck_for_path(&rel),
                path: rel,
                source,
            });
        }
    }
    Ok(())
}

fn start_github_load(
    repo: String,
    git_ref: String,
    token: String,
    mut project: Signal<Project>,
    mut active_file: Signal<Option<String>>,
) {
    spawn(async move {
        let loaded =
            match fetch_github_workspace_async(repo.trim(), git_ref.trim(), token.trim()).await {
                Ok(workspace) if workspace.files.is_empty() => build_project_with_packages(
                    repo.to_string(),
                    workspace.files,
                    workspace.package_files,
                    "GitHub repo loaded, but no .typ files were found.".to_string(),
                ),
                Ok(workspace) => build_project_with_packages(
                    format!("github:{repo}@{git_ref}"),
                    workspace.files,
                    workspace.package_files,
                    "Loaded GitHub workspace.".to_string(),
                ),
                Err(err) => Project {
                    message: format!("GitHub load failed: {err}"),
                    ..Project::sample()
                },
            };
        active_file.set(loaded.active_file.clone());
        project.set(loaded);
    });
}

#[derive(Default)]
struct GitHubWorkspace {
    files: Vec<TypstFile>,
    package_files: Vec<TypstPackageFile>,
}

#[derive(Deserialize)]
struct GitTreeResponse {
    tree: Vec<GitTreeEntry>,
}

#[derive(Deserialize)]
struct GitTreeEntry {
    path: String,
    #[serde(rename = "type")]
    kind: String,
    url: String,
}

#[derive(Deserialize)]
struct GitBlob {
    content: String,
    encoding: String,
}

#[cfg(not(target_arch = "wasm32"))]
async fn fetch_github_workspace_async(
    repo: &str,
    git_ref: &str,
    token: &str,
) -> Result<GitHubWorkspace, String> {
    let files = match fetch_github_archive_files_async(repo, git_ref, token).await {
        Ok(files) => files,
        Err(_) => fetch_github_files_async(repo, git_ref, token).await?,
    };
    Ok(GitHubWorkspace {
        files,
        package_files: Vec::new(),
    })
}

#[cfg(target_arch = "wasm32")]
async fn fetch_github_workspace_async(
    repo: &str,
    git_ref: &str,
    token: &str,
) -> Result<GitHubWorkspace, String> {
    let files = fetch_github_files_async(repo, git_ref, token).await?;
    let package_files = fetch_preview_packages_for_files(&files).await?;
    Ok(GitHubWorkspace {
        files,
        package_files,
    })
}

#[cfg(not(target_arch = "wasm32"))]
async fn fetch_github_archive_files_async(
    repo: &str,
    git_ref: &str,
    token: &str,
) -> Result<Vec<TypstFile>, String> {
    if !repo.contains('/') {
        return Err("Repository must be owner/repo.".to_string());
    }
    let client = reqwest::Client::new();
    let url = format!("https://api.github.com/repos/{repo}/tarball/{git_ref}");
    let mut request = client.get(url).header("User-Agent", "typst-web");
    if !token.is_empty() {
        request = request.bearer_auth(token);
    }
    let bytes = request
        .send()
        .await
        .map_err(|err| err.to_string())?
        .error_for_status()
        .map_err(|err| err.to_string())?
        .bytes()
        .await
        .map_err(|err| err.to_string())?;
    typst_files_from_tar_gz(bytes.as_ref(), true)
}

#[cfg(target_arch = "wasm32")]
async fn fetch_github_archive_files_async(
    repo: &str,
    git_ref: &str,
    token: &str,
) -> Result<Vec<TypstFile>, String> {
    use gloo_net::http::Request;

    if !repo.contains('/') {
        return Err("Repository must be owner/repo.".to_string());
    }
    let url = format!("https://api.github.com/repos/{repo}/tarball/{git_ref}");
    let mut request = Request::get(&url).header("Accept", "application/vnd.github+json");
    if !token.is_empty() {
        request = request.header("Authorization", &format!("Bearer {token}"));
    }
    let bytes = request
        .send()
        .await
        .map_err(|err| err.to_string())?
        .binary()
        .await
        .map_err(|err| err.to_string())?;
    typst_files_from_tar_gz(&bytes, true)
}

fn typst_files_from_tar_gz(bytes: &[u8], strip_root: bool) -> Result<Vec<TypstFile>, String> {
    let decoder = flate2::read::GzDecoder::new(bytes);
    let mut archive = tar::Archive::new(decoder);
    let mut files = Vec::new();
    let entries = archive.entries().map_err(|err| err.to_string())?;
    for entry in entries {
        let mut entry = entry.map_err(|err| err.to_string())?;
        if !entry.header().entry_type().is_file() {
            continue;
        }
        let path = entry.path().map_err(|err| err.to_string())?;
        let mut components = path.components();
        if strip_root {
            components.next();
        }
        let rel_path = components.as_path();
        let Some(rel) = normalize_relative_path(rel_path) else {
            continue;
        };
        if !rel.ends_with(".typ") || has_ignored_segment(&rel) {
            continue;
        }
        let mut source = String::new();
        std::io::Read::read_to_string(&mut entry, &mut source).map_err(|err| err.to_string())?;
        files.push(TypstFile {
            deck: deck_for_path(&rel),
            path: rel,
            source,
        });
    }
    Ok(files)
}

#[cfg(not(target_arch = "wasm32"))]
async fn fetch_github_files_async(
    repo: &str,
    git_ref: &str,
    token: &str,
) -> Result<Vec<TypstFile>, String> {
    if !repo.contains('/') {
        return Err("Repository must be owner/repo.".to_string());
    }
    let client = reqwest::Client::new();
    let url = format!("https://api.github.com/repos/{repo}/git/trees/{git_ref}?recursive=1");
    let mut request = client.get(url).header("User-Agent", "typst-web");
    if !token.is_empty() {
        request = request.bearer_auth(token);
    }
    let tree: GitTreeResponse = request
        .send()
        .await
        .map_err(|err| err.to_string())?
        .error_for_status()
        .map_err(|err| err.to_string())?
        .json()
        .await
        .map_err(|err| err.to_string())?;
    let mut files = Vec::new();
    for entry in tree.tree {
        if entry.kind == "blob" && entry.path.ends_with(".typ") && !has_ignored_segment(&entry.path)
        {
            let mut blob_request = client.get(entry.url).header("User-Agent", "typst-web");
            if !token.is_empty() {
                blob_request = blob_request.bearer_auth(token);
            }
            let blob: GitBlob = blob_request
                .send()
                .await
                .map_err(|err| err.to_string())?
                .error_for_status()
                .map_err(|err| err.to_string())?
                .json()
                .await
                .map_err(|err| err.to_string())?;
            if blob.encoding != "base64" {
                continue;
            }
            let cleaned = blob.content.replace('\n', "");
            let bytes = base64_decode(&cleaned)?;
            let source = String::from_utf8(bytes).map_err(|err| err.to_string())?;
            files.push(TypstFile {
                deck: deck_for_path(&entry.path),
                path: entry.path,
                source,
            });
        }
    }
    Ok(files)
}

#[cfg(target_arch = "wasm32")]
async fn fetch_github_files_async(
    repo: &str,
    git_ref: &str,
    token: &str,
) -> Result<Vec<TypstFile>, String> {
    use gloo_net::http::Request;

    if !repo.contains('/') {
        return Err("Repository must be owner/repo.".to_string());
    }
    let url = format!("https://api.github.com/repos/{repo}/git/trees/{git_ref}?recursive=1");
    let mut request = Request::get(&url).header("Accept", "application/vnd.github+json");
    if !token.is_empty() {
        request = request.header("Authorization", &format!("Bearer {token}"));
    }
    let tree: GitTreeResponse = request
        .send()
        .await
        .map_err(|err| err.to_string())?
        .json()
        .await
        .map_err(|err| err.to_string())?;
    let typ_paths: Vec<String> = tree
        .tree
        .into_iter()
        .filter(|entry| {
            entry.kind == "blob"
                && entry.path.ends_with(".typ")
                && !has_ignored_segment(&entry.path)
        })
        .map(|entry| entry.path)
        .collect();

    if !token.is_empty() {
        let (owner, name) = repo
            .split_once('/')
            .ok_or_else(|| "Repository must be owner/repo.".to_string())?;
        return fetch_github_files_graphql_async(owner, name, git_ref, token, &typ_paths).await;
    }

    let mut files = Vec::new();
    for path in typ_paths {
        let blob_url = format!("https://api.github.com/repos/{repo}/contents/{path}?ref={git_ref}");
        let contents: GitHubContent = Request::get(&blob_url)
            .header("Accept", "application/vnd.github+json")
            .send()
            .await
            .map_err(|err| err.to_string())?
            .json()
            .await
            .map_err(|err| err.to_string())?;
        if contents.encoding != "base64" {
            continue;
        }
        let cleaned = contents.content.replace('\n', "");
        let bytes = base64_decode(&cleaned)?;
        let source = String::from_utf8(bytes).map_err(|err| err.to_string())?;
        files.push(TypstFile {
            deck: deck_for_path(&path),
            path,
            source,
        });
    }
    Ok(files)
}

#[cfg(target_arch = "wasm32")]
#[derive(Deserialize)]
struct GitHubContent {
    content: String,
    encoding: String,
}

#[cfg(target_arch = "wasm32")]
async fn fetch_github_files_graphql_async(
    owner: &str,
    name: &str,
    git_ref: &str,
    token: &str,
    paths: &[String],
) -> Result<Vec<TypstFile>, String> {
    use gloo_net::http::Request;
    use serde_json::{Map, Value, json};

    let mut files = Vec::new();
    for chunk in paths.chunks(40) {
        let mut query = String::from("query($owner:String!, $name:String!,");
        for (index, _) in chunk.iter().enumerate() {
            query.push_str(&format!("$expr{index}: String!,"));
        }
        query.pop();
        query.push_str("){ repository(owner:$owner, name:$name){");
        for (index, _) in chunk.iter().enumerate() {
            query.push_str(&format!(
                "f{index}: object(expression:$expr{index}){{ ... on Blob {{ text }} }}"
            ));
        }
        query.push_str("}}");

        let mut variables = Map::new();
        variables.insert("owner".to_string(), json!(owner));
        variables.insert("name".to_string(), json!(name));
        for (index, path) in chunk.iter().enumerate() {
            variables.insert(format!("expr{index}"), json!(format!("{git_ref}:{path}")));
        }

        let response: Value = Request::post("https://api.github.com/graphql")
            .header("Accept", "application/vnd.github+json")
            .header("Authorization", &format!("Bearer {token}"))
            .json(&json!({ "query": query, "variables": variables }))
            .map_err(|err| err.to_string())?
            .send()
            .await
            .map_err(|err| err.to_string())?
            .json()
            .await
            .map_err(|err| err.to_string())?;

        if let Some(errors) = response.get("errors") {
            return Err(errors.to_string());
        }
        let repo = response
            .get("data")
            .and_then(|data| data.get("repository"))
            .ok_or_else(|| "GitHub GraphQL response missing repository.".to_string())?;
        for (index, path) in chunk.iter().enumerate() {
            let key = format!("f{index}");
            let text = repo
                .get(&key)
                .and_then(|entry| entry.get("text"))
                .and_then(|text| text.as_str())
                .ok_or_else(|| format!("Missing file contents for {path}"))?;
            files.push(TypstFile {
                deck: deck_for_path(path),
                path: path.clone(),
                source: text.to_string(),
            });
        }
    }
    Ok(files)
}

fn has_ignored_segment(path: &str) -> bool {
    path.split('/')
        .any(|part| part.starts_with('.') || part == "node_modules")
}

fn shared_root_prefix(files: &[TypstFile]) -> Option<String> {
    let mut first = files.first()?.path.split('/');
    let mut prefix = String::new();
    let Some(candidate) = first.next() else {
        return None;
    };
    if candidate.is_empty() {
        return None;
    }
    for file in files.iter().skip(1) {
        let Some(head) = file.path.split('/').next() else {
            return None;
        };
        if head != candidate {
            return None;
        }
    }
    prefix.push_str(candidate);
    Some(prefix)
}

#[cfg(target_arch = "wasm32")]
async fn fetch_preview_packages_for_files(
    files: &[TypstFile],
) -> Result<Vec<TypstPackageFile>, String> {
    use gloo_net::http::Request;

    let mut seen = BTreeSet::new();
    let mut pending = preview_packages_in_files(files);
    let mut package_files = Vec::new();

    while let Some(package) = pending.pop_first() {
        if !seen.insert(package.clone()) {
            continue;
        }

        let url = format!(
            "https://packages.typst.org/preview/{}-{}.tar.gz",
            package.name, package.version
        );
        let response = Request::get(&url)
            .send()
            .await
            .map_err(|err| format!("Failed to fetch package {package}: {err}"))?;
        if !response.ok() {
            return Err(format!(
                "Failed to fetch package {package}: HTTP {}",
                response.status()
            ));
        }
        let bytes = response.binary().await.map_err(|err| err.to_string())?;
        let files = typst_package_files_from_tar_gz(
            &bytes,
            &package.namespace,
            &package.name,
            &package.version,
        )?;
        pending.extend(
            preview_packages_in_package_files(&files)
                .difference(&seen)
                .cloned(),
        );
        package_files.extend(files);
    }
    Ok(package_files)
}

#[cfg(any(target_arch = "wasm32", test))]
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct PreviewPackage {
    namespace: String,
    name: String,
    version: String,
}

#[cfg(target_arch = "wasm32")]
impl std::fmt::Display for PreviewPackage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "@{}/{}:{}", self.namespace, self.name, self.version)
    }
}

#[cfg(target_arch = "wasm32")]
fn preview_packages_in_files(files: &[TypstFile]) -> BTreeSet<PreviewPackage> {
    preview_packages_in_sources(files.iter().map(|file| file.source.as_str()))
}

#[cfg(target_arch = "wasm32")]
fn preview_packages_in_package_files(files: &[TypstPackageFile]) -> BTreeSet<PreviewPackage> {
    preview_packages_in_sources(files.iter().filter_map(|file| file.source.as_deref()))
}

#[cfg(any(target_arch = "wasm32", test))]
fn preview_packages_in_sources<'a>(
    sources: impl IntoIterator<Item = &'a str>,
) -> BTreeSet<PreviewPackage> {
    let package_re = Regex::new(r#"@preview/([A-Za-z0-9][A-Za-z0-9-]*):([0-9]+\.[0-9]+\.[0-9]+)"#)
        .expect("valid regex");
    sources
        .into_iter()
        .flat_map(|source| {
            package_re
                .captures_iter(source)
                .map(|cap| PreviewPackage {
                    namespace: "preview".to_string(),
                    name: cap[1].to_string(),
                    version: cap[2].to_string(),
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

#[cfg(any(target_arch = "wasm32", test))]
fn typst_package_files_from_tar_gz(
    bytes: &[u8],
    namespace: &str,
    name: &str,
    version: &str,
) -> Result<Vec<TypstPackageFile>, String> {
    let decoder = flate2::read::GzDecoder::new(bytes);
    let mut archive = tar::Archive::new(decoder);
    let mut raw_files = Vec::new();
    let entries = archive.entries().map_err(|err| err.to_string())?;
    for entry in entries {
        let mut entry = entry.map_err(|err| err.to_string())?;
        if !entry.header().entry_type().is_file() {
            continue;
        }
        let path = entry.path().map_err(|err| err.to_string())?;
        let Some(rel) = normalize_relative_path(&path) else {
            continue;
        };
        let mut bytes = Vec::new();
        std::io::Read::read_to_end(&mut entry, &mut bytes).map_err(|err| err.to_string())?;
        raw_files.push((rel, bytes));
    }

    let archive_root =
        package_archive_root_segment(raw_files.iter().map(|(path, _)| path.as_str()));
    let mut files = Vec::new();
    for (path, bytes) in raw_files.clone() {
        let rel = if let Some(segment) = &archive_root {
            path.strip_prefix(segment)
                .unwrap_or(&path)
                .trim_start_matches('/')
        } else {
            &path
        };
        if rel.is_empty() {
            continue;
        }
        let source = String::from_utf8(bytes.clone()).ok();
        files.push(TypstPackageFile {
            namespace: namespace.to_string(),
            name: name.to_string(),
            version: version.to_string(),
            path: rel.to_string(),
            bytes,
            source,
        });
    }
    Ok(files)
}

#[cfg(any(target_arch = "wasm32", test))]
fn package_archive_root_segment<'a>(paths: impl IntoIterator<Item = &'a str>) -> Option<String> {
    let mut root = None;
    let mut has_nested_manifest = false;

    for path in paths {
        let (head, tail) = path.split_once('/')?;
        if head.is_empty() || tail.is_empty() {
            return None;
        }
        match &root {
            Some(root) if root != head => return None,
            Some(_) => {}
            None => root = Some(head.to_string()),
        }
        if tail == "typst.toml" {
            has_nested_manifest = true;
        }
    }

    has_nested_manifest
        .then_some(root?)
        .filter(|root| root != "src")
}

fn base64_decode(input: &str) -> Result<Vec<u8>, String> {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD
        .decode(input)
        .map_err(|err| err.to_string())
}

fn card_typst_file(card: &Flashcard, side: CardSide) -> TypstFile {
    let path = card_typst_path(card, side);
    let source = format!(
        "#set page(width: 360pt, height: auto, margin: 10pt)\n#set text(size: 12pt)\n{}",
        side.source(card)
    );
    TypstFile {
        path,
        deck: card.deck.clone(),
        source,
    }
}

fn card_typst_path(card: &Flashcard, side: CardSide) -> String {
    let directory = Path::new(&card.source_path)
        .parent()
        .and_then(|path| normalize_relative_path(path))
        .unwrap_or_default();
    let name = format!(".typst-web-card-{}-{}.typ", card.id, side.label());
    if directory.is_empty() {
        name
    } else {
        format!("{directory}/{name}")
    }
}

#[cfg(target_arch = "wasm32")]
fn web_fonts() -> Vec<typst::text::Font> {
    WEB_FONT_CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        cache
            .get_or_insert_with(|| {
                WEB_FONTS
                    .iter()
                    .flat_map(|bytes| {
                        typst::text::Font::iter(typst::foundations::Bytes::new(*bytes))
                    })
                    .collect()
            })
            .clone()
    })
}

fn render_typst_svg(project: &Project, file: &TypstFile) -> Result<String, String> {
    use typst::layout::PagedDocument;
    use typst::syntax::{
        FileId, VirtualPath,
        package::{PackageSpec, PackageVersion},
    };
    use typst_as_lib::TypstEngine;

    let root_prefix = shared_root_prefix(&project.files);
    let mut source_paths: BTreeMap<String, String> = BTreeMap::new();
    for file in &project.files {
        source_paths.insert(file.path.clone(), file.source.clone());
        if let Some(prefix) = &root_prefix {
            if let Some(stripped) = file
                .path
                .strip_prefix(prefix)
                .and_then(|rest| rest.strip_prefix('/'))
            {
                if !stripped.is_empty() {
                    source_paths.insert(stripped.to_string(), file.source.clone());
                }
            }
        }
    }
    source_paths.insert(file.path.clone(), file.source.clone());
    let mut sources: Vec<(FileId, String)> = source_paths
        .into_iter()
        .map(|(path, source)| (FileId::new(None, VirtualPath::new(&path)), source))
        .collect();
    for package_file in &project.package_files {
        if !package_file.path.ends_with(".typ") {
            continue;
        }
        let Some(source) = &package_file.source else {
            continue;
        };
        let version = package_file
            .version
            .parse::<PackageVersion>()
            .map_err(|err| err.to_string())?;
        let package = PackageSpec {
            namespace: package_file.namespace.clone().into(),
            name: package_file.name.clone().into(),
            version,
        };
        sources.push((
            FileId::new(Some(package), VirtualPath::new(&package_file.path)),
            source.clone(),
        ));
    }
    let source_refs: Vec<(FileId, &str)> = sources
        .iter()
        .map(|(id, source)| (*id, source.as_str()))
        .collect();
    let binaries: Vec<(FileId, &[u8])> = project
        .package_files
        .iter()
        .map(|package_file| {
            let version = package_file
                .version
                .parse::<PackageVersion>()
                .map_err(|err| err.to_string())?;
            let package = PackageSpec {
                namespace: package_file.namespace.clone().into(),
                name: package_file.name.clone().into(),
                version,
            };
            Ok((
                FileId::new(Some(package), VirtualPath::new(&package_file.path)),
                package_file.bytes.as_slice(),
            ))
        })
        .collect::<Result<_, String>>()?;
    let builder = TypstEngine::builder()
        .with_static_source_file_resolver(source_refs)
        .with_static_file_resolver(binaries);
    #[cfg(not(target_arch = "wasm32"))]
    let builder = builder
        .with_file_system_resolver(".")
        .with_package_file_resolver();
    #[cfg(target_arch = "wasm32")]
    let builder = builder.fonts(web_fonts());
    #[cfg(not(target_arch = "wasm32"))]
    let builder =
        builder.search_fonts_with(typst_as_lib::typst_kit_options::TypstKitFontOptions::default());
    let engine = builder.build();
    let warned = engine.compile::<_, PagedDocument>(file.path.as_str());
    match warned.output {
        Ok(document) => Ok(typst_svg::svg_merged(
            &document,
            typst::layout::Abs::pt(8.0),
        )),
        Err(err) => Err(format!("{err:?}")),
    }
}

fn cached_render_typst_svg(project: &Project, file: &TypstFile) -> Result<String, String> {
    let key = render_cache_key(project, file);
    if let Some(rendered) = RENDER_CACHE.with(|cache| cache.borrow().get(&key).cloned()) {
        return rendered;
    }

    let rendered = render_typst_svg(project, file);
    RENDER_CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        if cache.len() > 96 {
            cache.clear();
        }
        cache.insert(key, rendered.clone());
    });
    rendered
}

fn render_cache_key(project: &Project, file: &TypstFile) -> String {
    let mut hasher = Sha256::new();
    hasher.update(project.source_label.as_bytes());
    hasher.update(file.path.as_bytes());
    hasher.update(file.source.as_bytes());
    for package_file in &project.package_files {
        hasher.update(package_file.namespace.as_bytes());
        hasher.update(package_file.name.as_bytes());
        hasher.update(package_file.version.as_bytes());
        hasher.update(package_file.path.as_bytes());
        hasher.update(&package_file.bytes);
    }
    format!("{:x}", hasher.finalize())
}

#[cfg(not(target_arch = "wasm32"))]
fn load_progress_for(source_label: &str) -> Option<SrsProgress> {
    let path = progress_path(source_label)?;
    std::fs::read_to_string(path)
        .ok()
        .and_then(|raw| serde_json::from_str(&raw).ok())
}

#[cfg(target_arch = "wasm32")]
fn load_progress_for(_source_label: &str) -> Option<SrsProgress> {
    None
}

#[cfg(not(target_arch = "wasm32"))]
fn persist_progress(project: &Project) {
    let Some(path) = progress_path(&project.source_label) else {
        return;
    };
    let progress = project.progress.clone();
    spawn(async move {
        let _ = tokio::task::spawn_blocking(move || {
            if let Ok(raw) = serde_json::to_string_pretty(&progress) {
                let _ = std::fs::write(path, raw);
            }
        })
        .await;
    });
}

#[cfg(not(target_arch = "wasm32"))]
fn progress_path(source_label: &str) -> Option<PathBuf> {
    if source_label.starts_with("github:") || source_label == "sample workspace" {
        return None;
    }
    Some(PathBuf::from(source_label).join(".srs-progress.json"))
}

#[cfg(target_arch = "wasm32")]
fn persist_progress(_project: &Project) {}

fn deck_for_path(path: &str) -> String {
    Path::new(path)
        .parent()
        .and_then(|parent| normalize_relative_path(parent))
        .filter(|path| !path.is_empty())
        .unwrap_or_else(|| "root".to_string())
}

fn file_name(path: &str) -> String {
    Path::new(path)
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| path.to_string())
}

fn card_count_for(cards: &[Flashcard], path: &str) -> usize {
    cards.iter().filter(|card| card.source_path == path).count()
}

fn excerpt(source: &str) -> String {
    let clean = source.lines().take(8).collect::<Vec<_>>().join("\n");
    if clean.len() > 420 {
        format!("{}...", &clean[..420])
    } else {
        clean
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn package_archive_root_segment_preserves_package_root_paths() {
        let paths = ["typst.toml", "src/lib.typ", "src/util.typ"];

        assert_eq!(package_archive_root_segment(paths), None);
    }

    #[test]
    fn package_archive_root_segment_strips_archive_wrapper() {
        let paths = [
            "package-0.1.0/typst.toml",
            "package-0.1.0/src/lib.typ",
            "package-0.1.0/assets/icon.svg",
        ];

        assert_eq!(
            package_archive_root_segment(paths),
            Some("package-0.1.0".to_string())
        );
    }

    #[test]
    fn typst_package_files_from_tar_gz_preserves_package_paths() {
        let bytes = package_tar_gz([
            (
                "typst.toml",
                r#"[package]
name = "pkg"
version = "0.1.0"
entrypoint = "src/lib.typ"
"#,
            ),
            ("src/lib.typ", "#let value = 1"),
        ]);

        let files =
            typst_package_files_from_tar_gz(&bytes, "preview", "pkg", "0.1.0").expect("package");
        let paths = files
            .into_iter()
            .map(|file| file.path)
            .collect::<BTreeSet<_>>();

        assert!(paths.contains("typst.toml"));
        assert!(paths.contains("src/lib.typ"));
    }

    #[test]
    fn preview_packages_in_sources_finds_transitive_imports() {
        let packages = preview_packages_in_sources([
            r#"#import "@preview/first:0.1.0": foo"#,
            r#"#import "@preview/second:2.3.4": bar"#,
        ]);

        assert!(packages.contains(&PreviewPackage {
            namespace: "preview".to_string(),
            name: "first".to_string(),
            version: "0.1.0".to_string(),
        }));
        assert!(packages.contains(&PreviewPackage {
            namespace: "preview".to_string(),
            name: "second".to_string(),
            version: "2.3.4".to_string(),
        }));
    }

    #[test]
    fn render_resolves_transitive_static_packages() {
        let project = build_project_with_packages(
            "test".to_string(),
            vec![TypstFile {
                path: "main.typ".to_string(),
                deck: "root".to_string(),
                source: r#"#import "@preview/rootpkg:0.1.0": value
#value()"#
                    .to_string(),
            }],
            vec![
                test_package_file(
                    "rootpkg",
                    "typst.toml",
                    r#"[package]
name = "rootpkg"
version = "0.1.0"
entrypoint = "src/lib.typ"
"#,
                ),
                test_package_file(
                    "rootpkg",
                    "src/lib.typ",
                    r#"#import "@preview/deppkg:0.1.0": dep
#let value() = dep()"#,
                ),
                test_package_file(
                    "deppkg",
                    "typst.toml",
                    r#"[package]
name = "deppkg"
version = "0.1.0"
entrypoint = "src/lib.typ"
"#,
                ),
                test_package_file("deppkg", "src/lib.typ", "#let dep() = [ok]"),
            ],
            "test".to_string(),
        );
        let file = project.files.first().expect("main file");

        render_typst_svg(&project, file).expect("transitive package render should compile");
    }

    #[test]
    fn render_card_typst_file_compiles_markup() {
        let project = Project::sample();
        let card = Flashcard {
            id: "card".to_string(),
            deck: "root".to_string(),
            source_path: "index.typ".to_string(),
            question: "#strong[What is $x^2$?]".to_string(),
            answer: "A #emph[Typst] expression.".to_string(),
        };
        let file = card_typst_file(&card, CardSide::Question);

        let svg = render_typst_svg(&project, &file).expect("card markup should render");
        assert!(svg.contains("<svg"));
    }

    fn test_package_file(name: &str, path: &str, source: &str) -> TypstPackageFile {
        TypstPackageFile {
            namespace: "preview".to_string(),
            name: name.to_string(),
            version: "0.1.0".to_string(),
            path: path.to_string(),
            bytes: source.as_bytes().to_vec(),
            source: Some(source.to_string()),
        }
    }

    fn package_tar_gz<const N: usize>(files: [(&str, &str); N]) -> Vec<u8> {
        let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
        {
            let mut archive = tar::Builder::new(&mut encoder);
            for (path, source) in files {
                let mut header = tar::Header::new_gnu();
                header.set_size(source.len() as u64);
                header.set_mode(0o644);
                header.set_cksum();
                archive
                    .append_data(&mut header, path, source.as_bytes())
                    .expect("append tar entry");
            }
            archive.finish().expect("finish tar");
        }
        encoder.finish().expect("finish gzip")
    }
}
