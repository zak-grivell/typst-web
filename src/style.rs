pub(crate) const CSS: &str = r#"
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
.app { height: 100dvh; overflow: hidden; }
.screen { height: 100dvh; display: grid; grid-template-rows: auto minmax(0, 1fr); overflow: hidden; }
.screen-header {
  min-height: 54px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: calc(8px + env(safe-area-inset-top)) 14px 8px;
  background: rgba(41, 44, 60, 0.92);
  border-bottom: 1px solid rgba(198, 208, 245, 0.12);
}
.screen-body { min-height: 0; overflow: auto; padding: 14px; -webkit-overflow-scrolling: touch; }
.header-action { min-height: 34px; padding: 6px 10px; }
.brand { display: flex; align-items: center; gap: 10px; min-width: 0; }
.mark { width: 18px; height: 18px; border-radius: 50%; background: linear-gradient(135deg, var(--blue), var(--mauve)); box-shadow: 0 0 18px rgba(140,170,238,.45); }
.title { font-weight: 800; white-space: nowrap; }
.subtitle { color: var(--muted); font-size: 12px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.panel { background: rgba(65, 69, 89, 0.46); border: 1px solid rgba(198, 208, 245, 0.12); border-radius: 8px; padding: 14px; }
.stack { display: grid; gap: 12px; }
.file-list { display: grid; gap: 8px; }
.row { display: flex; gap: 8px; align-items: center; flex-wrap: wrap; }
.file-row { width: 100%; text-align: left; display: grid; gap: 3px; }
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
.metrics { display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); gap: 8px; }
.metrics button { display: grid; gap: 2px; justify-items: start; min-height: 72px; }
.metric-value { font-size: 24px; font-weight: 800; color: white; }
.metric-label { color: var(--muted); font-size: 12px; }
@media (max-width: 900px) {
  body { overflow: hidden; }
  .app { height: 100dvh; min-height: 100dvh; overflow: hidden; }
  .screen { height: 100dvh; }
  .screen-header { min-height: 48px; padding: calc(7px + env(safe-area-inset-top)) 10px 7px; }
  .screen-body { padding: 10px; }
  .brand { gap: 8px; }
  .mark { width: 16px; height: 16px; flex: 0 0 auto; }
  .subtitle { max-width: 100%; }
  .panel { padding: 11px; }
  .source-panel { gap: 8px; }
  .source-panel input, .source-panel button { min-height: 42px; }
  .file-row { min-height: 48px; padding: 8px 10px; }
  .file-name { font-size: 14px; }
  .muted { font-size: 12px; }
  .pill { max-width: 100%; overflow-wrap: anywhere; }
  .svg-page { max-height: 68dvh; padding: 10px; }
  .source { font-size: 12px; line-height: 1.45; }
  .card-render { padding: 8px; }
  .graph { min-height: 420px; }
  .node-list { grid-template-columns: 1fr; }
  .ratings { grid-template-columns: 1fr 1fr; }
  .metrics { grid-template-columns: 1fr 1fr 1fr; }
  .metrics button { min-height: 64px; padding: 8px; }
}
@media (max-width: 520px) {
  button { min-height: 42px; padding: 8px 10px; }
  input, textarea, select { min-height: 42px; padding: 9px 10px; }
  .title { font-size: 15px; }
  .row { gap: 6px; }
  .ratings { gap: 8px; }
}
"#;
