# typst-web Application Summary

`typst-web` is a no-server Dioxus application for working with Typst note projects across desktop, mobile, and web targets. It replaces the previous Bun-hosted `typst-notes` command set with a single Rust app that owns the UI, project indexing, preview, spaced repetition, graph exploration, and persistence in-process.

The app is local-first. It can load a Typst project from the local file system on desktop/mobile builds, or from a public/private GitHub repository on web-capable builds. All project analysis runs in the app process. There is no companion HTTP server, no server functions, and no remote rendering service.

## Runtime And Packaging

- Runtime: Rust with Dioxus `0.7`.
- UI targets: web, desktop, and mobile through Dioxus feature flags.
- Typst rendering: embedded Rust Typst engine where available, with SVG output shown directly inside the Dioxus view.
- State storage: local project files on native targets and browser/mobile storage on sandboxed targets.
- Network access: only direct GitHub API/raw-content requests initiated by the client when a repository source is selected.
- Optional Nix packaging remains available through `flake.nix`.

Cargo features:

```toml
[features]
default = ["desktop"]
web = ["dioxus/web"]
desktop = ["dioxus/desktop"]
mobile = ["dioxus/mobile"]
```

The package builds one executable named `typst-web`.

## Source Model

The application has a `Workspace` abstraction that hides where files came from:

- `Local`: a user-selected or typed project directory on native builds.
- `GitHub`: a repository specified as `owner/repo`, branch/ref, and optional token.

Each workspace produces the same in-memory model:

```rust
Project {
  source_label,
  files: Vec<TypstFile>,
  graph: GraphModel,
  decks: Vec<Deck>,
  progress: SrsProgress,
  active_file: Option<String>,
}
```

`TypstFile` contains:

- slash-separated relative path
- containing directory/deck name
- UTF-8 source text
- last modified timestamp when available

File discovery:

- Recursively scans for `.typ` files.
- Ignores hidden path segments.
- Ignores `node_modules`.
- Sorts paths alphabetically.
- Treats root files as deck `root`; nested files use their containing directory as the deck.
- Excludes `setup.typ` from graph nodes while keeping it available to Typst imports.

## GitHub Loading

GitHub sources are loaded directly by the app:

- Public repositories use unauthenticated GitHub contents/tree requests.
- Private repositories require a personal access token entered by the user.
- Tokens are used only for the current load request and are not written to project files.
- The app fetches a Git tree recursively for the selected ref, filters `.typ` blobs, downloads file content, and builds the same project model as local scans.
- No Typst source or token is sent to an application-owned server.

## Layout And Navigation

The app is a single Dioxus interface with four modes:

- `Preview`
- `SRS`
- `Graph`
- `Files`

The shell is compact and utility-focused:

- Sticky top toolbar with brand, source selector, global search, and mode tabs.
- Dark Catppuccin-inspired palette carried over from the original summary.
- Responsive layout for mobile, tablet, desktop, and narrow web views.
- Touch-friendly controls on SRS and graph interactions.
- No landing page; the first screen is the usable project tool.

## Preview Mode

Preview mode displays the selected `.typ` document without a server.

Features:

- Directory/file browser for the loaded project.
- Fuzzy file search with keyboard and touch support.
- Embedded SVG preview for the active Typst document.
- Multi-page output is shown as stacked page frames when the renderer returns multiple pages.
- Compile diagnostics are displayed inline with the source path and error text.
- Source fallback is available when the current platform cannot run the embedded renderer; this is the current web behavior.
- Active file tracking is internal to the app rather than LSP/SSE based.

The old `/follow` and server reload behavior is replaced with app state:

- Selecting a file updates `active_file`.
- External native file reload can rescan the project.
- GitHub projects can be refreshed from the selected ref.
- Mobile/web snapshots are refreshed by re-importing or refetching.

## SRS Mode

SRS mode keeps the original flashcard workflow but runs fully in the app.

Flashcard discovery:

- Scans `.typ` source for `flashcard(q, a)` calls.
- Extracts question and answer markup from balanced parentheses and quoted/content arguments where possible.
- Generates deterministic card ids from deck, source path, and question text.
- Groups cards by deck.

Review modes:

- Due: default queue containing cards with `next_review <= now`.
- All: every discovered card.
- Cram: every card, with per-session hiding after rating.
- Redo: cards whose previous rating was not Easy.

Scheduling:

- Uses the SM-2 style algorithm from the original app.
- Ratings are Again `1`, Hard `3`, Good `4`, Easy `5`.
- Ease factor is floored at `1.3`.
- Progress stores interval, repetitions, next review, last review, and last quality.

Persistence:

- Native local projects store `.srs-progress.json` next to the project.
- GitHub/web/mobile persistence is modeled in app state; durable browser/mobile storage is the next implementation step.
- The app never writes back to GitHub automatically.

UI:

- Full-height review frame.
- Progress pill and progress track.
- Question side first, then `Show Answer`.
- Rating buttons: Again, Hard, Good, Easy.
- Keyboard shortcuts on desktop/web: Space, 1, 2, 3, 4.

## Graph Mode

Graph mode builds a note-link graph in-process.

Graph construction:

- Scans all `.typ` files except `setup.typ`.
- Extracts quoted relative `.typ` links such as `./file.typ` and `../topic/file.typ`.
- Resolves links relative to the source file directory.
- Keeps edges only when the target exists in the loaded workspace.
- Counts inbound, outbound, and total degree for every node.

UI:

- Canvas/SVG-style force graph rendered by Dioxus.
- Search focuses a node by path or basename.
- Selected node displays path, deck, inbound, outbound, and degree.
- Nodes can be inspected on touch or pointer devices.
- Layout is deterministic enough for repeatable sessions but still spreads dense projects.

## Files Mode

Files mode exposes the loaded project model directly:

- Lists all `.typ` files by deck.
- Shows source excerpts.
- Allows selecting a file for preview.
- Shows derived card counts and link counts.
- Provides refresh controls for local and GitHub workspaces.

## Data And Persistence Files

Native local workspace files:

- `.srs-progress.json`: SRS scheduling state.
- `.typst-web/active-file.json`: last active file.

Sandboxed targets:

- Use app-local storage keyed by workspace source label in a future persistence adapter.
- GitHub tokens are never persisted by the current implementation.

Progress shape:

```json
{
  "cards": {
    "<card-id>": {
      "id": "<card-id>",
      "deck": "<deck>",
      "ease_factor": 2.5,
      "interval": 0,
      "repetitions": 0,
      "next_review": 0,
      "last_review": null,
      "last_quality": null
    }
  }
}
```

## Important Implementation Details

- Use Dioxus components and signals for all UI state.
- Do not use Dioxus server functions.
- Do not run a Bun, Axum, or other application server.
- Keep source loading behind the workspace abstraction.
- Keep project scanning, graph building, flashcard extraction, and SM-2 scheduling as pure Rust logic where possible.
- Keep all paths slash-separated and relative to the selected workspace root.
- Treat GitHub and local projects identically after loading.
- Prefer embedded Typst rendering; show explicit diagnostics/fallbacks when platform limitations prevent it.
