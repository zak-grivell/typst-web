use crate::domain::{Project, SavedGitHubRepo, SrsProgress};

#[cfg(not(target_arch = "wasm32"))]
use dioxus::prelude::spawn;
#[cfg(not(target_arch = "wasm32"))]
use std::path::PathBuf;

#[cfg(target_arch = "wasm32")]
const SAVED_GITHUB_REPOS_KEY: &str = "typst-web:saved-github-repos";

pub(crate) fn load_saved_github_repos() -> Vec<SavedGitHubRepo> {
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

pub(crate) fn remember_github_repo(
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

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn load_progress_for(source_label: &str) -> Option<SrsProgress> {
    let path = progress_path(source_label)?;
    std::fs::read_to_string(path)
        .ok()
        .and_then(|raw| serde_json::from_str(&raw).ok())
}

#[cfg(target_arch = "wasm32")]
pub(crate) fn load_progress_for(_source_label: &str) -> Option<SrsProgress> {
    None
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn persist_progress(project: &Project) {
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
pub(crate) fn persist_progress(_project: &Project) {}
