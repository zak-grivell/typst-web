use dioxus::prelude::*;
use std::collections::{BTreeSet, HashMap};
use std::path::Path;

mod domain;
mod project;
mod render;
mod storage;
mod style;
mod workspace;

use domain::{Flashcard, GraphNode, Mode, Project, ReviewState, SrsProgress, TypstFile};
use project::{card_count_for, file_name, filter_files, normalize_relative_path};
use render::cached_render_typst_svg;
use storage::{load_saved_github_repos, persist_progress, remember_github_repo};
use style::CSS;
use workspace::{start_github_load, start_local_load};

#[cfg(test)]
use domain::TypstPackageFile;
#[cfg(test)]
use project::build_project_with_packages;
#[cfg(test)]
use render::render_typst_svg;
#[cfg(test)]
use workspace::{
    PreviewPackage, package_archive_root_segment, preview_packages_in_sources,
    typst_package_files_from_tar_gz,
};

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let mode = use_signal(|| Mode::Workspace);
    let project = use_signal(Project::sample);
    let active_file = use_signal(|| project.read().active_file.clone());
    let query = use_signal(String::new);
    let local_path = use_signal(String::new);
    let github_repo = use_signal(String::new);
    let github_ref = use_signal(|| "main".to_string());
    let github_token = use_signal(String::new);
    let srs = use_signal(SrsSession::default);

    let project_value = project.read().clone();

    rsx! {
        style { "{CSS}" }
        div { class: "app",
            match *mode.read() {
                Mode::Workspace => rsx! {
                    div { class: "screen",
                        ScreenHeader { title: "Workspace", subtitle: project_value.source_label.clone(), mode }
                        div { class: "screen-body stack",
                            SourcePanel {
                                local_path,
                                github_repo,
                                github_ref,
                                github_token,
                                project,
                                active_file
                            }
                            WorkspaceSummary { project: project_value.clone(), mode }
                        }
                    }
                },
                Mode::Files => rsx! {
                    div { class: "screen",
                        ScreenHeader { title: "Files", subtitle: project_value.source_label.clone(), mode }
                        FilesScreen { project: project_value.clone(), query, active_file, mode }
                    }
                },
                Mode::Preview => rsx! {
                    div { class: "screen",
                        ScreenHeader {
                            title: "Preview",
                            subtitle: active_file.read().clone().unwrap_or_else(|| "No file selected".to_string()),
                            mode
                        }
                        div { class: "screen-body",
                            PreviewView { project: project_value.clone(), active_file: active_file.read().clone() }
                        }
                    }
                },
                Mode::Srs => rsx! {
                    div { class: "screen",
                        ScreenHeader { title: "SRS", subtitle: format!("{} cards", project_value.cards.len()), mode }
                        div { class: "screen-body",
                            SrsView { project, srs }
                        }
                    }
                },
                Mode::Graph => rsx! {
                    div { class: "screen",
                        ScreenHeader { title: "Graph", subtitle: format!("{} links", project_value.graph_edges.len()), mode }
                        div { class: "screen-body",
                            GraphView { project: project_value.clone() }
                        }
                    }
                },
            }
        }
    }
}

#[component]
fn ScreenHeader(title: &'static str, subtitle: String, mut mode: Signal<Mode>) -> Element {
    rsx! {
        header { class: "screen-header",
            div { class: "brand",
                div { class: "mark" }
                div {
                    div { class: "title", "{title}" }
                    div { class: "subtitle", "{subtitle}" }
                }
            }
            if *mode.read() != Mode::Workspace {
                button {
                    class: "header-action",
                    onclick: move |_| mode.set(Mode::Workspace),
                    "Workspace"
                }
            }
        }
    }
}

#[component]
fn WorkspaceSummary(project: Project, mut mode: Signal<Mode>) -> Element {
    rsx! {
        div { class: "panel stack",
            div { class: "metrics",
                button { onclick: move |_| mode.set(Mode::Files),
                    span { class: "metric-value", "{project.files.len()}" }
                    span { class: "metric-label", "Files" }
                }
                button { onclick: move |_| mode.set(Mode::Srs),
                    span { class: "metric-value", "{project.cards.len()}" }
                    span { class: "metric-label", "Cards" }
                }
                button { onclick: move |_| mode.set(Mode::Graph),
                    span { class: "metric-value", "{project.graph_edges.len()}" }
                    span { class: "metric-label", "Links" }
                }
            }
        }
    }
}

#[component]
fn FilesScreen(
    project: Project,
    mut query: Signal<String>,
    mut active_file: Signal<Option<String>>,
    mut mode: Signal<Mode>,
) -> Element {
    let filtered = filter_files(&project.files, &query.read());

    rsx! {
        div { class: "screen-body stack",
            input {
                value: "{query}",
                placeholder: "Search files...",
                oninput: move |event| query.set(event.value())
            }
            div { class: "file-list",
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
                        div { class: "muted", "{card_count_for(&project.cards, &file.path)} cards" }
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
