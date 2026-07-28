#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";

const args = new Map();
for (let i = 2; i < process.argv.length; i += 2) args.set(process.argv[i], process.argv[i + 1]);

const root = path.resolve(args.get("--root") ?? ".");
const out = path.resolve(args.get("--out") ?? path.join(root, "gallery/review/paperbark-trial"));
const commit = args.get("--commit") ?? "unknown";
const dirty = args.get("--dirty") === "true";
const profilesPath = path.join(root, "scripts/paperbark-trial/profiles.tsv");

const esc = (value) =>
  String(value)
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;");

const profiles = fs
  .readFileSync(profilesPath, "utf8")
  .split(/\r?\n/)
  .filter((line) => line.trim() && !line.startsWith("#"))
  .map((line, index) => {
    const fields = line.split("\t");
    if (fields.length !== 4) {
      throw new Error(`profiles.tsv:${index + 1}: expected 4 tab-separated fields, got ${fields.length}`);
    }
    const [id, slug, label, description] = fields;
    return { id, slug, label, description };
  });

if (profiles.length !== 5) throw new Error(`expected exactly five profiles, got ${profiles.length}`);
for (const field of ["id", "slug", "label"]) {
  const values = profiles.map((profile) => profile[field]);
  if (new Set(values).size !== 5) throw new Error(`Paperbark profiles have duplicate ${field}s`);
}
if (profiles.map(({ id }) => id).join("") !== "ABCDE") {
  throw new Error(`Paperbark profile order must be A–E, got ${profiles.map(({ id }) => id).join("")}`);
}

function requireFile(rel) {
  const absolute = path.join(out, rel);
  const stat = fs.statSync(absolute, { throwIfNoEntry: false });
  if (!stat?.isFile() || stat.size === 0) throw new Error(`missing or empty Paperbark artifact: ${rel}`);
  return absolute;
}

function pngDimensions(absolute) {
  const header = Buffer.alloc(24);
  const fd = fs.openSync(absolute, "r");
  try {
    if (fs.readSync(fd, header, 0, header.length, 0) !== header.length) {
      throw new Error(`short PNG header: ${absolute}`);
    }
  } finally {
    fs.closeSync(fd);
  }
  if (header.subarray(1, 4).toString("ascii") !== "PNG") throw new Error(`not a PNG: ${absolute}`);
  return [header.readUInt32BE(16), header.readUInt32BE(20)];
}

function normalizedSidecar(data) {
  const normalized = structuredClone(data);
  delete normalized.page.background;
  return normalized;
}

const expectedPngs = [];
const sidecars = new Map();
for (const widthClass of ["wide", "narrow"]) {
  for (const profile of profiles) {
    const stem = `${profile.id.toLowerCase()}-${profile.slug}-${widthClass}`;
    const png = `assets/${stem}.png`;
    const json = `assets/${stem}.json`;
    const pngPath = requireFile(png);
    const jsonPath = requireFile(json);
    expectedPngs.push(path.basename(png));
    const dims = pngDimensions(pngPath);
    const expectedDims = widthClass === "wide" ? [3600, 2000] : [1800, 1400];
    if (JSON.stringify(dims) !== JSON.stringify(expectedDims)) {
      throw new Error(`${stem}: dimensions ${dims.join("x")} != ${expectedDims.join("x")}`);
    }
    const data = JSON.parse(fs.readFileSync(jsonPath, "utf8"));
    if (data.theme?.name !== "Paperbark trial") {
      throw new Error(`${stem}: sidecar theme ${JSON.stringify(data.theme?.name)} != "Paperbark trial"`);
    }
    if (data.canvas?.dpi !== 2) throw new Error(`${stem}: sidecar dpi ${data.canvas?.dpi} != 2`);
    if (data.page?.background?.kind !== "paperbark-trial") {
      throw new Error(`${stem}: sidecar does not report the disposable Paperbark ground`);
    }
    if (data.page.background.profile !== profile.id || data.page.background.slug !== profile.slug) {
      throw new Error(`${stem}: sidecar profile identity does not match ${profile.id}/${profile.slug}`);
    }
    if (data.page.background.static !== true) throw new Error(`${stem}: trial profile is not reported static`);
    sidecars.set(`${widthClass}:${profile.id}`, data);
  }
  const baseline = JSON.stringify(normalizedSidecar(sidecars.get(`${widthClass}:A`)));
  for (const profile of profiles.slice(1)) {
    const actual = JSON.stringify(normalizedSidecar(sidecars.get(`${widthClass}:${profile.id}`)));
    if (actual !== baseline) {
      throw new Error(
        `${widthClass}: ${profile.id} changes sidecar state outside page.background; ` +
          "document/theme geometry, palette, type, caret, and chrome must be identical",
      );
    }
  }
}

const actualPngs = fs
  .readdirSync(path.join(out, "assets"))
  .filter((name) => name.endsWith(".png"))
  .sort();
if (actualPngs.length !== 10 || actualPngs.join("\n") !== expectedPngs.sort().join("\n")) {
  throw new Error(`expected exactly the ten declared PNGs, got ${JSON.stringify(actualPngs)}`);
}

const card = (profile, widthClass) => {
  const stem = `${profile.id.toLowerCase()}-${profile.slug}-${widthClass}`;
  const geometry = widthClass === "wide" ? "3600×2000 px · 2× · 1800×1000 logical" : "1800×1400 px · 2× · 900×700 logical";
  return `
    <article class="shot" id="${esc(widthClass)}-${esc(profile.id)}" data-profile="${esc(profile.id)}">
      <a class="image-link" href="assets/${esc(stem)}.png" target="_blank" rel="noreferrer">
        <img src="assets/${esc(stem)}.png" alt="Paperbark ${esc(profile.id)} · ${esc(profile.label)} · ${esc(widthClass)}" loading="lazy">
      </a>
      <div class="caption">
        <h3>${esc(profile.id)} · ${esc(profile.label)}</h3>
        <p>${esc(profile.description)}</p>
        <div class="meta">${esc(geometry)} · <a href="assets/${esc(stem)}.json">sidecar</a></div>
      </div>
    </article>`;
};

const wideCards = profiles.map((profile) => card(profile, "wide")).join("\n");
const narrowCards = profiles.map((profile) => card(profile, "narrow")).join("\n");
const html = `<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>awl · Paperbark material trial</title>
<style>
  :root { color-scheme:dark; --bg:#111210; --panel:#1b1d19; --ink:#ecece5; --muted:#9ca097; --line:#30342d; --accent:#e6a85c; }
  * { box-sizing:border-box; }
  body { margin:0; background:var(--bg); color:var(--ink); font:15px/1.45 ui-sans-serif,system-ui,-apple-system,sans-serif; }
  header { position:sticky; top:0; z-index:5; padding:18px 24px; background:color-mix(in srgb,var(--bg) 94%,transparent); backdrop-filter:blur(18px); border-bottom:1px solid var(--line); }
  h1 { margin:0 0 5px; font-size:20px; font-weight:650; }
  header p { margin:0; color:var(--muted); }
  main { padding:24px; }
  section { margin:0 0 44px; }
  h2 { font-size:18px; margin:0 0 14px; }
  .wide-grid { display:grid; grid-template-columns:repeat(5,minmax(0,1fr)); gap:12px; }
  .narrow-grid { display:grid; grid-template-columns:repeat(5,minmax(0,1fr)); gap:12px; }
  .shot { min-width:0; overflow:hidden; background:var(--panel); border:1px solid var(--line); border-radius:10px; }
  .image-link { display:block; background:#080908; }
  .shot img { display:block; width:100%; height:auto; max-height:640px; object-fit:contain; }
  .caption { padding:12px 14px 14px; }
  .caption h3 { margin:0 0 5px; font-size:15px; }
  .caption p { margin:0 0 7px; color:var(--muted); }
  .meta { color:var(--muted); font:12px/1.4 ui-monospace,SFMono-Regular,monospace; }
  .meta a { color:var(--accent); }
  .summary { margin:0 0 28px; padding:12px 14px; color:var(--muted); background:var(--panel); border:1px solid var(--line); border-radius:8px; }
  dialog.lightbox { width:100vw; max-width:none; height:100vh; max-height:none; margin:0; padding:0; border:0; background:transparent; color:var(--ink); overflow:hidden; }
  dialog.lightbox::backdrop { background:rgb(4 5 4 / .92); backdrop-filter:blur(8px); }
  .lightbox-shell { position:relative; display:grid; grid-template-columns:auto minmax(0,1fr) auto; align-items:center; gap:14px; width:100%; height:100%; padding:22px; }
  .lightbox figure { min-width:0; max-width:100%; max-height:100%; margin:0; display:flex; flex-direction:column; align-items:center; gap:10px; }
  .lightbox img { display:block; max-width:100%; max-height:calc(100vh - 86px); object-fit:contain; border-radius:5px; box-shadow:0 20px 70px rgb(0 0 0 / .5); }
  .lightbox figcaption { display:flex; gap:12px; width:100%; justify-content:center; color:var(--muted); }
  .lightbox figcaption strong { color:var(--ink); }
  .lightbox button { color:var(--ink); background:rgb(27 29 25 / .88); border:1px solid #4a5045; border-radius:999px; width:44px; height:44px; padding:0; font-size:22px; cursor:pointer; }
  .lightbox-close { position:absolute; z-index:1; top:18px; right:18px; }
  .lightbox-position { font:12px/1.4 ui-monospace,SFMono-Regular,monospace; }
  .provenance { color:${dirty ? "#ffb36b" : "var(--muted)"}; }
  @media (max-width:1100px) { .wide-grid,.narrow-grid { overflow-x:auto; grid-template-columns:repeat(5,minmax(260px,1fr)); } }
</style>
</head>
<body>
<header>
  <h1>Paperbark · five static material treatments</h1>
  <p class="provenance">real Awl captures · commit ${esc(commit)}${dirty ? " · dirty trial branch" : ""}</p>
</header>
<main>
  <aside class="summary">The document, provisional world, page width, zoom, caret, selection, and chrome are fixed. Only the procedural margin treatment changes. No treatment is ranked or recommended.</aside>
  <section id="wide"><h2>Wide · compare A–E together</h2><div class="wide-grid">${wideCards}</div></section>
  <section id="narrow"><h2>Narrow · responsive check</h2><div class="narrow-grid">${narrowCards}</div></section>
</main>
<dialog class="lightbox" id="lightbox" aria-modal="true" aria-labelledby="lightbox-title">
  <div class="lightbox-shell">
    <button class="lightbox-close" type="button" aria-label="Close preview">×</button>
    <button class="lightbox-prev" type="button" aria-label="Previous image">←</button>
    <figure><img id="lightbox-image" alt=""><figcaption><strong id="lightbox-title"></strong><span class="lightbox-position" id="lightbox-position" aria-live="polite"></span></figcaption></figure>
    <button class="lightbox-next" type="button" aria-label="Next image">→</button>
  </div>
</dialog>
<script>
  const lightbox = document.querySelector("#lightbox");
  const image = document.querySelector("#lightbox-image");
  const title = document.querySelector("#lightbox-title");
  const position = document.querySelector("#lightbox-position");
  const links = [...document.querySelectorAll(".image-link")];
  let index = 0;
  let invoker = null;
  const update = () => {
    const link = links[index];
    image.src = link.getAttribute("href");
    image.alt = link.querySelector("img").alt;
    title.textContent = link.closest(".shot").querySelector("h3").textContent;
    position.textContent = (index + 1) + " of " + links.length;
  };
  const step = (delta) => { index = (index + delta + links.length) % links.length; update(); };
  const close = () => { if (lightbox.open) lightbox.close(); };
  for (const link of links) {
    link.addEventListener("click", (event) => {
      if (event.button !== 0 || event.metaKey || event.ctrlKey || event.shiftKey || event.altKey) return;
      event.preventDefault();
      index = links.indexOf(link);
      invoker = link;
      update();
      lightbox.showModal();
      document.querySelector(".lightbox-close").focus();
    });
  }
  document.querySelector(".lightbox-close").addEventListener("click", close);
  document.querySelector(".lightbox-prev").addEventListener("click", () => step(-1));
  document.querySelector(".lightbox-next").addEventListener("click", () => step(1));
  lightbox.addEventListener("click", (event) => { if (event.target === lightbox || event.target.classList.contains("lightbox-shell")) close(); });
  lightbox.addEventListener("cancel", (event) => { event.preventDefault(); close(); });
  lightbox.addEventListener("close", () => { image.removeAttribute("src"); invoker?.focus(); });
  document.addEventListener("keydown", (event) => {
    if (!lightbox.open) return;
    if (event.key === "ArrowLeft") { event.preventDefault(); step(-1); }
    if (event.key === "ArrowRight") { event.preventDefault(); step(1); }
  });
</script>
</body>
</html>`;

if (/https?:\/\//i.test(html)) throw new Error("Paperbark review contains an external URL");
const inlineScript = html.match(/<script>\s*([\s\S]*?)<\/script>/)?.[1];
if (!inlineScript) throw new Error("Paperbark review contains no interaction script");
new Function(inlineScript);
fs.writeFileSync(path.join(out, "index.html"), html);

const cards = [...html.matchAll(/<article class="shot"[^>]+id="([^"]+)"/g)];
if (cards.length !== 10) throw new Error(`DOM card count ${cards.length} != 10`);
if (new Set(cards.map((match) => match[1])).size !== 10) throw new Error("review contains duplicate card ids");
if ((html.match(/class="wide-grid"/g) ?? []).length !== 1) throw new Error("review needs exactly one A–E primary row");
for (const profile of profiles) {
  for (const widthClass of ["wide", "narrow"]) {
    const stem = `${profile.id.toLowerCase()}-${profile.slug}-${widthClass}`;
    if (!html.includes(`href="assets/${stem}.png"`) || !html.includes(`src="assets/${stem}.png"`)) {
      throw new Error(`review target missing for ${profile.id}/${widthClass}`);
    }
  }
}
for (const hook of [
  '<dialog class="lightbox" id="lightbox" aria-modal="true"',
  'aria-label="Close preview"',
  'aria-label="Previous image"',
  'aria-label="Next image"',
  'event.key === "ArrowLeft"',
  'event.key === "ArrowRight"',
  'lightbox.addEventListener("cancel"',
  'invoker?.focus()',
]) {
  if (!html.includes(hook)) throw new Error(`Paperbark lightbox is missing required hook: ${hook}`);
}

console.log(`Paperbark review: ${path.join(out, "index.html")} (5 profiles, 10 cards)`);
