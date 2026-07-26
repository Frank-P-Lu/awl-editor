#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";

const args = new Map();
for (let i = 2; i < process.argv.length; i += 2) args.set(process.argv[i], process.argv[i + 1]);

const root = path.resolve(args.get("--root") ?? ".");
const out = path.resolve(args.get("--out") ?? path.join(root, "gallery/review"));
const commit = args.get("--commit") ?? "unknown";
const dirty = args.get("--dirty") === "true";
const scenesPath = path.join(root, "scripts/review/scenes.tsv");

const esc = (value) =>
  String(value)
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;");

function readScenes() {
  const ids = new Set();
  return fs
    .readFileSync(scenesPath, "utf8")
    .split(/\r?\n/)
    .filter((line) => line.trim() && !line.startsWith("#"))
    .map((line, index) => {
      const fields = line.split("\t");
      if (fields.length !== 10) {
        throw new Error(`scenes.tsv:${index + 1}: expected 10 tab-separated fields, got ${fields.length}`);
      }
      const [id, label, theme, canvas, measure, keys, fixture, captureMode, expect, description] = fields;
      if (ids.has(id)) throw new Error(`duplicate scene id: ${id}`);
      ids.add(id);
      if (!["normal", "popover", "diff"].includes(captureMode)) {
        throw new Error(`scenes.tsv:${index + 1}: unknown capture mode ${captureMode}`);
      }
      return { id, label, theme, canvas, measure, keys, fixture, captureMode, expect, description };
    });
}

function pngs(dir) {
  if (!fs.existsSync(dir)) throw new Error(`missing directory: ${dir}`);
  return fs
    .readdirSync(dir)
    .filter((name) => name.endsWith(".png"))
    .sort((a, b) => a.localeCompare(b));
}

function requireFile(rel) {
  const absolute = path.join(out, rel);
  const stat = fs.statSync(absolute, { throwIfNoEntry: false });
  if (!stat?.isFile() || stat.size === 0) throw new Error(`missing or empty review artifact: ${rel}`);
  return rel;
}

function readSidecar(rel) {
  return JSON.parse(fs.readFileSync(path.join(out, rel), "utf8"));
}

function valueAt(data, dottedPath) {
  return dottedPath.split(".").reduce((value, key) => value?.[key], data);
}

function assertSceneState(scene, data) {
  for (const assertion of scene.expect.split(",")) {
    const [dottedPath, rawExpected] = assertion.split("=");
    if (!dottedPath || rawExpected === undefined) {
      throw new Error(`scene ${scene.id} has invalid expectation: ${assertion}`);
    }
    const expected =
      rawExpected === "true" ? true : rawExpected === "false" ? false : rawExpected === "null" ? null : rawExpected;
    const actual = valueAt(data, dottedPath);
    if (actual !== expected) {
      throw new Error(
        `scene ${scene.id} expected ${dottedPath}=${JSON.stringify(expected)}, got ${JSON.stringify(actual)}`,
      );
    }
  }
}

const scenes = readScenes();
const roomNames = pngs(path.join(out, "assets/worlds/room"));
const frameNames = pngs(path.join(out, "assets/worlds/frame"));
if (roomNames.join("\n") !== frameNames.join("\n")) throw new Error("Room and Frame world rosters differ");

const capturedWorlds = roomNames.map((name) => name.slice(0, -4));
if (capturedWorlds.length === 0) throw new Error("world roster is empty");
const iconManifestPath = requireFile("assets/icons/manifest.json");
const iconWorlds = readSidecar(iconManifestPath).worlds?.map((world) => world.name);
if (
  !iconWorlds ||
  JSON.stringify([...iconWorlds].sort()) !== JSON.stringify([...capturedWorlds].sort())
) {
  throw new Error(`icon roster differs from capture roster: ${JSON.stringify(iconWorlds)}`);
}
const worlds = iconWorlds;

const worldModes = new Map();
for (const world of worlds) {
  for (const surface of ["room", "frame"]) {
    requireFile(`assets/worlds/${surface}/${world}.png`);
    const json = requireFile(`assets/worlds/${surface}/${world}.json`);
    const sidecar = readSidecar(json);
    const reported = sidecar?.theme?.name;
    if (reported !== world) throw new Error(`${surface}/${world} sidecar reports theme ${reported}`);
    const mode = sidecar?.theme?.mode;
    if (!["light", "dark"].includes(mode)) throw new Error(`${surface}/${world} reports invalid mode ${mode}`);
    if (worldModes.has(world) && worldModes.get(world) !== mode) {
      throw new Error(`Room and Frame mode differ for ${world}`);
    }
    worldModes.set(world, mode);
  }
}
const sceneModes = new Map();
for (const scene of scenes) {
  requireFile(`assets/scenes/${scene.id}.png`);
  const json = requireFile(`assets/scenes/${scene.id}.json`);
  const data = readSidecar(json);
  const reported = data?.theme?.name;
  if (reported !== scene.theme) {
    throw new Error(`scene ${scene.id} requested ${scene.theme} but sidecar reports ${reported}`);
  }
  const mode = data?.theme?.mode;
  if (!["light", "dark"].includes(mode)) throw new Error(`scene ${scene.id} reports invalid mode ${mode}`);
  sceneModes.set(scene.id, mode);
  assertSceneState(scene, data);
}
for (const name of ["shipped-light.png", "shipped-dark.png"]) {
  requireFile(`assets/icons/gallery/${name}`);
}
const card = ({ id, label, image, json, theme, mode, surface, description, meta = "" }) => `
  <article class="shot" data-world="${esc(theme)}" data-mode="${esc(mode)}" data-surface="${esc(surface)}" id="${esc(id)}">
    <a class="image-link" href="${esc(image)}" target="_blank" rel="noreferrer">
      <img src="${esc(image)}" alt="${esc(label)}" loading="lazy">
    </a>
    <div class="caption">
      <h3>${esc(label)}</h3>
      <p>${esc(description)}</p>
      <div class="meta">${esc(meta)}${json ? ` · <a href="${esc(json)}">sidecar</a>` : ""}</div>
    </div>
  </article>`;

const worldCards = (surface) =>
  worlds
    .map((world) =>
      card({
        id: `${surface}-${world}`,
        label: `${world} · ${surface === "room" ? "Room" : "Frame"}`,
        image: `assets/worlds/${surface}/${world}.png`,
        json: `assets/worlds/${surface}/${world}.json`,
        theme: world,
        mode: worldModes.get(world),
        surface,
        description:
          surface === "room"
            ? "Canonical WYSIWYG document with the caret parked away."
            : "The same room with the command palette summoned.",
        meta: world,
      }),
    )
    .join("\n");

const sceneCards = scenes
  .map((scene) =>
    card({
      id: `scene-${scene.id}`,
      label: scene.label,
      image: `assets/scenes/${scene.id}.png`,
      json: `assets/scenes/${scene.id}.json`,
      theme: scene.theme,
      mode: sceneModes.get(scene.id),
      surface: "journey",
      description: scene.description,
      meta: `${scene.theme} · ${scene.canvas} · keys: ${scene.keys}`,
    }),
  )
  .join("\n");

const iconCards = ["light", "dark"]
  .map((mode) =>
    card({
      id: `icons-${mode}`,
      label: `Shipped icons · ${mode} surface`,
      image: `assets/icons/gallery/shipped-${mode}.png`,
      json: "",
      theme: mode,
      mode,
      surface: "icons",
      description:
        "Every world at its assigned cursor silhouette, rendered natively down the Dock/app-switcher ladder.",
      meta: "theme-derived offline icon pipeline",
    }),
  )
  .join("\n");

const options = worlds.map((world) => `<option value="${esc(world)}">${esc(world)}</option>`).join("");
const html = `<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>awl visual review · ${esc(commit.slice(0, 12))}</title>
<style>
  :root { color-scheme:dark; --bg:#111210; --panel:#1b1d19; --ink:#ecece5; --muted:#9ca097; --line:#30342d; --accent:#e6a85c; }
  * { box-sizing:border-box; }
  html { scroll-behavior:smooth; }
  body { margin:0; background:var(--bg); color:var(--ink); font:15px/1.45 ui-sans-serif,system-ui,-apple-system,sans-serif; }
  header { position:sticky; top:0; z-index:5; padding:18px 24px; background:color-mix(in srgb,var(--bg) 94%,transparent); backdrop-filter:blur(18px); border-bottom:1px solid var(--line); }
  h1 { margin:0 0 5px; font-size:20px; font-weight:650; }
  header p { margin:0; color:var(--muted); }
  nav { display:flex; gap:8px; flex-wrap:wrap; margin-top:14px; align-items:center; }
  nav a, select, button { color:var(--ink); background:var(--panel); border:1px solid var(--line); border-radius:7px; padding:7px 10px; text-decoration:none; font:inherit; }
  button { cursor:pointer; }
  main { padding:24px; }
  section { scroll-margin-top:125px; margin:0 0 44px; }
  h2 { font-size:18px; margin:0 0 14px; }
  .grid { display:grid; grid-template-columns:repeat(auto-fill,minmax(360px,1fr)); gap:18px; }
  .shot { min-width:0; overflow:hidden; background:var(--panel); border:1px solid var(--line); border-radius:10px; }
  .image-link { display:block; background:#080908; }
  .shot img { display:block; width:100%; height:auto; max-height:640px; object-fit:contain; }
  .caption { padding:12px 14px 14px; }
  .caption h3 { margin:0 0 5px; font-size:15px; }
  .caption p { margin:0 0 7px; color:var(--muted); }
  .meta { color:var(--muted); font:12px/1.4 ui-monospace,SFMono-Regular,monospace; }
  .meta a { color:var(--accent); }
  .shot[hidden] { display:none; }
  .summary { margin:0 0 28px; padding:12px 14px; color:var(--muted); background:var(--panel); border:1px solid var(--line); border-radius:8px; }
  .summary strong { color:var(--ink); }
  dialog.lightbox { width:100vw; max-width:none; height:100vh; max-height:none; margin:0; padding:0; border:0; background:transparent; color:var(--ink); overflow:hidden; }
  dialog.lightbox::backdrop { background:rgb(4 5 4 / .92); backdrop-filter:blur(8px); }
  .lightbox-shell { position:relative; display:grid; grid-template-columns:auto minmax(0,1fr) auto; align-items:center; gap:14px; width:100%; height:100%; padding:22px; }
  .lightbox figure { min-width:0; max-width:100%; max-height:100%; margin:0; display:flex; flex-direction:column; align-items:center; gap:10px; }
  .lightbox img { display:block; max-width:100%; max-height:calc(100vh - 86px); object-fit:contain; border-radius:5px; box-shadow:0 20px 70px rgb(0 0 0 / .5); }
  .lightbox figcaption { display:flex; gap:12px; width:100%; justify-content:center; color:var(--muted); }
  .lightbox figcaption strong { color:var(--ink); }
  .lightbox button { color:var(--ink); background:rgb(27 29 25 / .88); border:1px solid #4a5045; border-radius:999px; width:44px; height:44px; padding:0; font-size:22px; cursor:pointer; }
  .lightbox button:hover, .lightbox button:focus-visible { border-color:var(--accent); outline:none; }
  .lightbox-close { position:absolute; z-index:1; top:18px; right:18px; }
  .lightbox-position { font:12px/1.4 ui-monospace,SFMono-Regular,monospace; }
  .provenance { color:${dirty ? "#ffb36b" : "var(--muted)"}; }
  @media (max-width:650px) {
    header,main{padding-left:12px;padding-right:12px}.grid{grid-template-columns:1fr}
    .lightbox-shell{grid-template-columns:1fr;padding:58px 10px 10px}.lightbox-prev,.lightbox-next{position:absolute;bottom:12px}.lightbox-prev{left:12px}.lightbox-next{right:12px}
    .lightbox img{max-height:calc(100vh - 130px)}.lightbox figcaption{padding:0 48px}
  }
</style>
</head>
<body>
<header>
  <h1>awl visual review</h1>
  <p class="provenance">commit ${esc(commit)}${dirty ? " · dirty working tree" : ""} · ${worlds.length} worlds · ${scenes.length} canonical scenes</p>
  <nav>
    <a href="#rooms">Rooms</a><a href="#frames">Frames</a><a href="#journeys">Important screens</a><a href="#icons">Icons</a>
    <select id="world"><option value="">All worlds</option>${options}</select>
    <select id="mode"><option value="">Light + dark</option><option value="light">Light</option><option value="dark">Dark</option></select>
    <select id="surface"><option value="">All surfaces</option><option value="room">Room</option><option value="frame">Frame</option><option value="journey">Important screens</option><option value="icons">Icons</option></select>
    <button id="clear" type="button">Clear filters</button>
  </nav>
</header>
<main>
  <aside class="summary">
    <strong>Build checks passed.</strong>
    Every declared capture reached its expected sidecar state; Room, Frame, and icon rosters agree; all local targets are non-empty; no network URL is present.
    Timing, live menu dispatch, native blur, and interaction feel remain live-only review surfaces.
  </aside>
  <section id="rooms"><h2>World Rooms</h2><div class="grid">${worldCards("room")}</div></section>
  <section id="frames"><h2>World Frames</h2><div class="grid">${worldCards("frame")}</div></section>
  <section id="journeys"><h2>Important screens</h2><div class="grid">${sceneCards}</div></section>
  <section id="icons"><h2>Shipped app icons</h2><div class="grid">${iconCards}</div></section>
</main>
<dialog class="lightbox" id="lightbox" aria-modal="true" aria-labelledby="lightbox-title">
  <div class="lightbox-shell">
    <button class="lightbox-close" type="button" aria-label="Close preview">×</button>
    <button class="lightbox-prev" type="button" aria-label="Previous image">←</button>
    <figure>
      <img id="lightbox-image" alt="">
      <figcaption>
        <strong id="lightbox-title"></strong>
        <span class="lightbox-position" id="lightbox-position" aria-live="polite"></span>
      </figcaption>
    </figure>
    <button class="lightbox-next" type="button" aria-label="Next image">→</button>
  </div>
</dialog>
<script>
  const world = document.querySelector("#world");
  const mode = document.querySelector("#mode");
  const surface = document.querySelector("#surface");
  const lightbox = document.querySelector("#lightbox");
  const lightboxImage = document.querySelector("#lightbox-image");
  const lightboxTitle = document.querySelector("#lightbox-title");
  const lightboxPosition = document.querySelector("#lightbox-position");
  let lightboxLinks = [];
  let lightboxIndex = 0;
  let lightboxInvoker = null;
  const apply = () => {
    for (const shot of document.querySelectorAll(".shot")) {
      const worldMatch = !world.value || shot.dataset.world === world.value;
      const modeMatch = !mode.value || shot.dataset.mode === mode.value;
      const surfaceMatch = !surface.value || shot.dataset.surface === surface.value;
      shot.hidden = !(worldMatch && modeMatch && surfaceMatch);
    }
  };
  const visibleImageLinks = () =>
    [...document.querySelectorAll(".shot:not([hidden]) .image-link")];
  const updateLightbox = () => {
    const link = lightboxLinks[lightboxIndex];
    const card = link.closest(".shot");
    const thumbnail = link.querySelector("img");
    lightboxImage.src = link.getAttribute("href");
    lightboxImage.alt = thumbnail.alt;
    lightboxTitle.textContent = card.querySelector("h3").textContent;
    lightboxPosition.textContent = (lightboxIndex + 1) + " of " + lightboxLinks.length;
    const only = lightboxLinks.length < 2;
    document.querySelector(".lightbox-prev").disabled = only;
    document.querySelector(".lightbox-next").disabled = only;
  };
  const stepLightbox = (delta) => {
    lightboxIndex = (lightboxIndex + delta + lightboxLinks.length) % lightboxLinks.length;
    updateLightbox();
  };
  const closeLightbox = () => {
    if (lightbox.open) lightbox.close();
  };
  for (const link of document.querySelectorAll(".image-link")) {
    link.addEventListener("click", (event) => {
      if (event.button !== 0 || event.metaKey || event.ctrlKey || event.shiftKey || event.altKey) return;
      event.preventDefault();
      lightboxLinks = visibleImageLinks();
      lightboxIndex = lightboxLinks.indexOf(link);
      lightboxInvoker = link;
      updateLightbox();
      lightbox.showModal();
      document.querySelector(".lightbox-close").focus();
    });
  }
  document.querySelector(".lightbox-close").addEventListener("click", closeLightbox);
  document.querySelector(".lightbox-prev").addEventListener("click", () => stepLightbox(-1));
  document.querySelector(".lightbox-next").addEventListener("click", () => stepLightbox(1));
  lightbox.addEventListener("click", (event) => {
    if (event.target === lightbox || event.target.classList.contains("lightbox-shell")) closeLightbox();
  });
  lightbox.addEventListener("cancel", (event) => { event.preventDefault(); closeLightbox(); });
  lightbox.addEventListener("close", () => {
    lightboxImage.removeAttribute("src");
    lightboxInvoker?.focus();
  });
  document.addEventListener("keydown", (event) => {
    if (!lightbox.open) return;
    if (event.key === "ArrowLeft") { event.preventDefault(); stepLightbox(-1); }
    if (event.key === "ArrowRight") { event.preventDefault(); stepLightbox(1); }
  });
  world.addEventListener("change", apply);
  mode.addEventListener("change", apply);
  surface.addEventListener("change", apply);
  document.querySelector("#clear").addEventListener("click", () => { world.value=""; mode.value=""; surface.value=""; apply(); });
</script>
</body>
</html>`;

if (/https?:\/\//i.test(html)) throw new Error("dashboard contains an external URL");
const inlineScript = html.match(/<script>\s*([\s\S]*?)<\/script>/)?.[1];
if (!inlineScript) throw new Error("dashboard contains no interaction script");
try {
  new Function(inlineScript);
} catch (error) {
  throw new Error(`dashboard interaction script does not parse: ${error.message}`);
}
fs.writeFileSync(path.join(out, "index.html"), html);

const expectedCards = worlds.length * 2 + scenes.length + 2;
const actualCards = (html.match(/<article class="shot"/g) ?? []).length;
if (actualCards !== expectedCards) throw new Error(`DOM card count ${actualCards} != manifest count ${expectedCards}`);
const domIds = [...html.matchAll(/<article class="shot"[^>]+id="([^"]+)"/g)].map((match) => match[1]);
if (new Set(domIds).size !== domIds.length) throw new Error("dashboard contains duplicate card ids");
for (const hook of [
  '<dialog class="lightbox" id="lightbox" aria-modal="true"',
  'aria-label="Close preview"',
  'aria-label="Previous image"',
  'aria-label="Next image"',
  'event.key === "ArrowLeft"',
  'event.key === "ArrowRight"',
  'lightbox.addEventListener("cancel"',
  'lightboxInvoker?.focus()',
]) {
  if (!html.includes(hook)) throw new Error(`dashboard lightbox is missing required hook: ${hook}`);
}
console.log(`dashboard: ${path.join(out, "index.html")} (${actualCards} cards)`);
