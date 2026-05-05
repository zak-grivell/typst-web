use crate::domain::{Project, TypstFile};
use crate::workspace::shared_root_prefix;
use sha2::{Digest, Sha256};
use std::cell::RefCell;
use std::collections::BTreeMap;

#[cfg(target_arch = "wasm32")]
const WEB_FONTS: &[&[u8]] = &[
    include_bytes!("../assets/fonts/LibertinusSerif-Regular.otf"),
    include_bytes!("../assets/fonts/LibertinusSerif-Bold.otf"),
    include_bytes!("../assets/fonts/LibertinusSerif-Italic.otf"),
    include_bytes!("../assets/fonts/NewCMMath-Regular.otf"),
];

thread_local! {
    static RENDER_CACHE: RefCell<BTreeMap<String, Result<String, String>>> = RefCell::new(BTreeMap::new());
    #[cfg(target_arch = "wasm32")]
    static WEB_FONT_CACHE: RefCell<Option<Vec<typst::text::Font>>> = const { RefCell::new(None) };
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

pub(crate) fn render_typst_svg(project: &Project, file: &TypstFile) -> Result<String, String> {
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

pub(crate) fn cached_render_typst_svg(
    project: &Project,
    file: &TypstFile,
) -> Result<String, String> {
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
