use crate::domain::{
    Flashcard, GraphEdge, GraphNode, Project, ReviewState, TypstFile, TypstPackageFile,
};
use crate::storage::load_progress_for;
use regex::Regex;
use sha2::{Digest, Sha256};
use std::collections::{BTreeSet, HashMap};
use std::path::{Component, Path};

impl Project {
    pub(crate) fn sample() -> Self {
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

pub(crate) fn build_project(
    source_label: String,
    files: Vec<TypstFile>,
    message: String,
) -> Project {
    build_project_with_packages(source_label, files, Vec::new(), message)
}

pub(crate) fn build_project_with_packages(
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

pub(crate) fn filter_files(files: &[TypstFile], query: &str) -> Vec<TypstFile> {
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

pub(crate) fn normalize_relative_path(path: &Path) -> Option<String> {
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

pub(crate) fn deck_for_path(path: &str) -> String {
    Path::new(path)
        .parent()
        .and_then(|parent| normalize_relative_path(parent))
        .filter(|path| !path.is_empty())
        .unwrap_or_else(|| "root".to_string())
}

pub(crate) fn file_name(path: &str) -> String {
    Path::new(path)
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| path.to_string())
}

pub(crate) fn card_count_for(cards: &[Flashcard], path: &str) -> usize {
    cards.iter().filter(|card| card.source_path == path).count()
}
