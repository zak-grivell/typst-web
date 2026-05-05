use crate::domain::{Project, TypstFile, TypstPackageFile};
use crate::project::{
    build_project, build_project_with_packages, deck_for_path, normalize_relative_path,
};
use dioxus::prelude::*;
#[cfg(any(target_arch = "wasm32", test))]
use regex::Regex;
use serde::Deserialize;
#[cfg(any(target_arch = "wasm32", test))]
use std::collections::BTreeSet;
#[cfg(not(target_arch = "wasm32"))]
use std::path::Path;
#[cfg(not(target_arch = "wasm32"))]
use std::path::PathBuf;

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
pub(crate) fn start_local_load(
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
pub(crate) fn start_local_load(
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

pub(crate) fn start_github_load(
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

pub(crate) fn shared_root_prefix(files: &[TypstFile]) -> Option<String> {
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
pub(crate) struct PreviewPackage {
    pub(crate) namespace: String,
    pub(crate) name: String,
    pub(crate) version: String,
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
pub(crate) fn preview_packages_in_sources<'a>(
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
pub(crate) fn typst_package_files_from_tar_gz(
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
pub(crate) fn package_archive_root_segment<'a>(
    paths: impl IntoIterator<Item = &'a str>,
) -> Option<String> {
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

pub(crate) fn base64_decode(input: &str) -> Result<Vec<u8>, String> {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD
        .decode(input)
        .map_err(|err| err.to_string())
}
