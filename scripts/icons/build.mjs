#!/usr/bin/env node
// THE PAGE BUILDER — manifest + tuning + bundled font files -> self-contained
// HTML pages, plus the shot index the CDP driver replays.
//
// OFFLINE BY CONSTRUCTION: every font is inlined into one `fonts.css` as a
// base64 `data:` URL read straight off `assets/fonts/*.ttf`. No CDN, no
// webfont service, no network of any kind — the pages render identically with
// the machine in airplane mode, which is the only way an icon pipeline is
// allowed to work in a repo whose zero-network invariant is a design law.
//
// DETERMINISM: nothing here reads a clock, a random number, or the environment
// beyond its declared inputs; every iteration is over an explicitly ordered
// list. Same manifest + same tuning + same font files => byte-identical HTML =>
// byte-identical PNGs (verified by re-render in scripts/export-icons.sh).
//
// NO PER-WORLD BRANCHES: a world contributes four colors and a family name.
// Everything else comes from the preset (three of them) and the per-FAMILY
// tuning delta. `assertNoWorldKeys` makes "just special-case Wagtail" fail loudly.

import fs from "node:fs";
import path from "node:path";

const SIZES = [16, 24, 32, 44, 56, 64, 128, 256, 512, 1024];
// The study's own dock checks (it drew at 512 and looked at 44 / 24 / 56),
// carried forward so the candidates are judged the way the directions were.
const DOCK_SIZES = [56, 44, 24];
const SIZE_ROW = [256, 128, 64, 56, 44, 32, 24, 16];
const GAP = 16;

// The gallery sheets' two contexts: a dark dock and a light dock. Neutral
// greys on purpose — a world's icon has to hold up against a surface it does
// not control, which is exactly what a Dock is.
const SURFACES = {
  dark: { page: "#141416", strip: "#2a2a2e", text: "#e9e9ec", dim: "#9a9aa2", rule: "#3a3a40" },
  light: { page: "#f0f0f2", strip: "#ffffff", text: "#1a1a1c", dim: "#6a6a72", rule: "#d8d8de" },
};

const esc = (s) =>
  String(s).replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;");

// ---------------------------------------------------------------- tuning ---

function assertNoWorldKeys(tuning, manifest) {
  const families = new Set(manifest.faces.map((f) => f.family));
  const worlds = new Set(manifest.worlds.map((w) => w.name));
  const presetNames = new Set(Object.keys(tuning.presets));
  for (const key of Object.keys(tuning.faces)) {
    if (worlds.has(key)) {
      throw new Error(
        `tuning.json keys a WORLD (${key}). Tuning is per FACE only — if a world needs its own numbers, the lockup is wrong.`
      );
    }
    if (!families.has(key)) {
      throw new Error(`tuning.json keys ${key}, which no shipped world wears`);
    }
    const face = tuning.faces[key];
    for (const k of Object.keys(face)) {
      if (k === "presets" || k.startsWith("_")) continue; // "_seat" etc: inline documentation, JSON has no comments
      if (!tuning.allowed.includes(k)) {
        throw new Error(`tuning.json: ${key}.${k} is not one of ${tuning.allowed.join("/")}`);
      }
    }
    // The SEAT override: a face may carry an additional delta that only
    // applies to worlds wearing this face AT ONE PRESET. This is what makes
    // Bitter (Mopoke=pill, Magpie=block) and Iosevka (Currawong=pill,
    // Cassowary=block) correctable independently without a world key — the
    // override is keyed by the shape the base already varies by, not by
    // world. Absent `presets`, or an absent entry for the preset actually in
    // play, composes to zero: every face that doesn't need this is untouched.
    for (const presetName of Object.keys(face.presets ?? {})) {
      if (!presetNames.has(presetName)) {
        throw new Error(
          `tuning.json: ${key}.presets.${presetName} is not one of ${[...presetNames].join("/")}`
        );
      }
      const override = face.presets[presetName];
      for (const k of Object.keys(override)) {
        if (!tuning.allowed.includes(k)) {
          throw new Error(`tuning.json: ${key}.presets.${presetName}.${k} is not one of ${tuning.allowed.join("/")}`);
        }
      }
    }
  }
}

function clamp(v, [lo, hi], what) {
  if (typeof v !== "number" || !Number.isFinite(v)) throw new Error(`${what} must be a number, got ${v}`);
  if (v < lo || v > hi) throw new Error(`${what} = ${v} is outside the allowed ${lo}..${hi}`);
  return v;
}

/** Compose preset + per-family delta + per-(family,preset) seat override into
 * the four numbers a tile needs. The seat override is the SAME bounded delta
 * mechanism as the flat face delta, just scoped one level narrower — it lands
 * on top of the face's flat delta rather than replacing it, so a face keeps
 * exactly one flat tuning plus, only where named, one further correction for
 * the single preset that needs it. */
function geometry(tuning, preset, family) {
  const p = tuning.presets[preset];
  const face = tuning.faces[family] ?? {};
  const seat = (face.presets ?? {})[preset] ?? {};
  const add = (key) => {
    const d = (face[key] ?? 0) + (seat[key] ?? 0);
    clamp(d, tuning.bounds.delta, `${family}[.presets.${preset}].${key} combined delta`);
    return clamp(p[key] + d, tuning.bounds.final, `${family}.${key} final`);
  };
  let radius = seat.radius ?? face.radius ?? p.radius;
  if (p.radius === "capsule") radius = "capsule";
  if (radius !== "capsule") clamp(radius, tuning.bounds.radius, `${family}.radius`);
  const weight = seat.weight ?? face.weight ?? "regular";
  if (!tuning.bounds.weight.includes(weight)) {
    throw new Error(`tuning.json: ${family}.weight must be regular|bold, got ${weight}`);
  }
  return {
    ix: add("insetX"),
    it: add("insetTop"),
    ib: add("insetBottom"),
    sy: clamp((face.seatY ?? 0) + (seat.seatY ?? 0), tuning.bounds.delta, `${family}[.presets.${preset}].seatY combined delta`),
    radius: radius === "capsule" ? "999px" : `${radius}em`,
    weight,
  };
}

// ------------------------------------------------------------------ tile ---

function tileStyle(world, face, geom, size) {
  const file = geom.weight === "bold" && face.bold ? face.bold : face.regular;
  return [
    `--s:${size}px`,
    `--ground:${world.ground}`,
    `--ink:${world.base_content}`,
    `--cursor:${world.primary}`,
    `--curink:${world.primary_content}`,
    `--face:'${world.font}'`,
    `--fw:${file.weight}`,
    `--ix:${geom.ix}%`,
    `--it:${geom.it}%`,
    `--ib:${geom.ib}%`,
    `--sy:${geom.sy}%`,
    `--r:${geom.radius}`,
  ].join(";");
}

function tile(world, face, geom, size, id) {
  const idAttr = id ? ` id="${esc(id)}"` : "";
  return `<div class="tile"${idAttr} style="${tileStyle(world, face, geom, size)}"><span class="mark">aw<span class="cur">l</span></span></div>`;
}

// ------------------------------------------------------------- font css ---

function fontsCss(manifest, fontsDir) {
  const out = [
    "/* GENERATED by scripts/icons/build.mjs — every face inlined from the repo's",
    "   own assets/fonts, at the weight the FILE declares. Nothing is fetched. */",
  ];
  const seen = new Set();
  for (const face of manifest.faces) {
    for (const file of [face.regular, face.bold]) {
      if (!file || seen.has(`${face.family}|${file.file}`)) continue;
      seen.add(`${face.family}|${file.file}`);
      const bytes = fs.readFileSync(path.join(fontsDir, file.file));
      out.push(
        `@font-face { font-family: "${face.family}"; font-style: normal; font-weight: ${file.weight};`,
        `  src: url(data:font/ttf;base64,${bytes.toString("base64")}) format("truetype"); }`
      );
    }
  }
  return out.join("\n") + "\n";
}

// ----------------------------------------------------------------- pages ---

const CHROME_CSS = `
body { margin: 0; font-family: "IBM Plex Sans"; -webkit-font-smoothing: antialiased; }
/* The sheet is SHRINK-TO-FIT: a gallery page is exactly as wide as its widest
 * row, so the captured PNG has no dead margin and its width is a fact of the
 * content rather than of whatever viewport the renderer happened to open. */
.page { display: inline-block; }
.sheet { padding: 32px; }
h1 { font-size: 22px; font-weight: 600; margin: 0 0 4px; letter-spacing: 0.01em; }
.sub { font-size: 13px; margin: 0 0 24px; }
.strip { border-radius: 18px; padding: 20px 24px; margin-bottom: 20px; }
.row { display: flex; align-items: flex-end; gap: 18px; }
.cell { display: flex; flex-direction: column; align-items: center; gap: 8px; }
.cap { font-size: 11px; letter-spacing: 0.04em; text-transform: uppercase; }
.world { font-size: 14px; font-weight: 600; width: 96px; }
.grid { display: grid; gap: 18px 22px; align-items: center; }
.hdr { font-size: 11px; letter-spacing: 0.06em; text-transform: uppercase; }
.legend { font-size: 12px; margin-top: 18px; }
`;

function page({ title, surface, body, transparent }) {
  const s = SURFACES[surface] ?? SURFACES.dark;
  const bg = transparent ? "transparent" : s.page;
  return `<!doctype html>
<html><head><meta charset="utf-8"><title>${esc(title)}</title>
<link rel="stylesheet" href="fonts.css"><link rel="stylesheet" href="lockup.css">
<style>${CHROME_CSS}
body { background: ${bg}; color: ${s.text}; }
.sub, .cap, .hdr, .legend { color: ${s.dim}; }
.strip { background: ${s.strip}; }
.rule { border-top: 1px solid ${s.rule}; margin: 18px 0; }
</style></head><body><div class="page">${body}</div></body></html>`;
}

/** One tiles page per size: the export sheet the .icns is eventually cut from. */
function tilesPage(manifest, tuning, size, faceOf) {
  const cols = size >= 512 ? 6 : Math.max(1, Math.min(8, Math.floor(1600 / (size + GAP))));
  const shots = [];
  let html = "";
  let i = 0;
  for (const world of manifest.worlds) {
    for (const preset of Object.keys(tuning.presets)) {
      const x = GAP + (i % cols) * (size + GAP);
      const y = GAP + Math.floor(i / cols) * (size + GAP);
      const geom = geometry(tuning, preset, world.font);
      html += `<div style="position:absolute;left:${x}px;top:${y}px">${tile(world, faceOf(world.font), geom, size)}</div>`;
      shots.push({ out: `tiles/${world.name}-${preset}-${size}.png`, x, y, w: size, h: size });
      i += 1;
    }
  }
  const rows = Math.ceil(i / cols);
  const w = GAP + cols * (size + GAP);
  const h = GAP + rows * (size + GAP);
  return {
    file: `tiles-${size}.html`,
    transparent: true,
    viewport: { w: Math.min(w, 2000), h: Math.min(h, 2000) },
    shots,
    html: page({
      title: `awl icon tiles @${size}`,
      surface: "dark",
      transparent: true,
      body: `<div style="position:relative;width:${w}px;height:${h}px">${html}</div>`,
    }),
  };
}

/** Every candidate at one glance: 19 worlds x 3 presets, on one dock surface. */
function overviewPage(manifest, tuning, surface, faceOf) {
  const presets = Object.keys(tuning.presets);
  let rows = `<div class="grid" style="grid-template-columns: 110px repeat(${presets.length}, auto); justify-content: start">`;
  rows += `<div></div>` + presets.map((p) => `<div class="hdr">${esc(tuning.presets[p].label)}</div>`).join("");
  for (const world of manifest.worlds) {
    rows += `<div class="world">${esc(world.name)}<div class="cap" style="font-weight:400">${esc(world.font)}</div></div>`;
    for (const preset of presets) {
      const geom = geometry(tuning, preset, world.font);
      rows += `<div>${tile(world, faceOf(world.font), geom, 128)}</div>`;
    }
  }
  rows += `</div>`;
  return {
    file: `overview-${surface}.html`,
    surface,
    shots: [{ out: `gallery/overview-${surface}.png`, full: true }],
    html: page({
      title: `awl icon candidates — ${surface}`,
      surface,
      body: `<div class="sheet"><h1>awl app icons — every candidate</h1>
<p class="sub">${manifest.worlds.length} worlds &times; 3 logo-cursor presets, at 128px, on a ${surface} surface. Colors are the world's own ground / base_content / primary / primary_content.</p>
<div class="strip">${rows}</div></div>`,
    }),
  };
}

/** One preset, every world, down the size roster — the "survives the dock" sheet. */
function sizesPage(manifest, tuning, preset, surface, faceOf) {
  let rows = `<div class="grid" style="grid-template-columns: 110px repeat(${SIZE_ROW.length}, auto); justify-content: start">`;
  rows += `<div></div>` + SIZE_ROW.map((s) => `<div class="hdr">${s}px</div>`).join("");
  for (const world of manifest.worlds) {
    rows += `<div class="world">${esc(world.name)}</div>`;
    const geom = geometry(tuning, preset, world.font);
    for (const size of SIZE_ROW) {
      rows += `<div>${tile(world, faceOf(world.font), geom, size)}</div>`;
    }
  }
  rows += `</div>`;
  return {
    file: `sizes-${preset}-${surface}.html`,
    surface,
    shots: [{ out: `gallery/sizes-${preset}-${surface}.png`, full: true }],
    html: page({
      title: `awl icon sizes — ${preset} — ${surface}`,
      surface,
      body: `<div class="sheet"><h1>Preset: ${esc(tuning.presets[preset].label)}</h1>
<p class="sub">Every world down the macOS size roster, each rendered natively at that pixel size (not scaled down from the master). ${surface} surface.</p>
<div class="strip">${rows}</div></div>`,
    }),
  };
}

/** A literal Dock row: all 19 worlds at 56 / 44 / 24 px, one preset. */
function dockPage(manifest, tuning, preset, surface, faceOf) {
  let body = "";
  for (const size of DOCK_SIZES) {
    const icons = manifest.worlds
      .map((w) => tile(w, faceOf(w.font), geometry(tuning, preset, w.font), size))
      .join("");
    body += `<div class="strip" style="display:flex;width:max-content;align-items:center;gap:${Math.max(6, Math.round(size / 5))}px">${icons}</div>
<p class="cap" style="margin:-10px 0 18px">${size}px</p>`;
  }
  return {
    file: `dock-${preset}-${surface}.html`,
    surface,
    shots: [{ out: `gallery/dock-${preset}-${surface}.png`, full: true }],
    html: page({
      title: `awl dock strip — ${preset} — ${surface}`,
      surface,
      body: `<div class="sheet"><h1>Dock strip — ${esc(tuning.presets[preset].label)}</h1>
<p class="sub">The three sizes the study checked: 56px dock, 44px dock, 24px app-switcher/menu. ${surface} surface.</p>${body}</div>`,
    }),
  };
}

/** WHAT ACTUALLY SHIPS: each world at the ONE preset its `Theme` assigns.
 *
 * The assignment is not repeated here — it rides in on the manifest's `cursor`
 * field, straight off `worlds.rs`'s `icon_cursor`, which a new world cannot
 * compile without filling in. So this sheet cannot show a stale roster: it
 * shows whatever the worlds currently declare, or it fails to build. */
function shippedPage(manifest, tuning, surface, faceOf) {
  const shown = [256, 128, 64, 44, 32, 24];
  let rows = `<div class="grid" style="grid-template-columns: 150px repeat(${shown.length}, auto); justify-content: start">`;
  rows += `<div></div>` + shown.map((s) => `<div class="hdr">${s}px</div>`).join("");
  for (const world of manifest.worlds) {
    if (!tuning.presets[world.cursor]) {
      throw new Error(`${world.name} declares cursor ${world.cursor}, which is not one of the three presets`);
    }
    const geom = geometry(tuning, world.cursor, world.font);
    rows += `<div class="world" style="width:150px">${esc(world.name)}<div class="cap" style="font-weight:400">${esc(
      tuning.presets[world.cursor].label
    )}</div></div>`;
    for (const size of shown) rows += `<div>${tile(world, faceOf(world.font), geom, size)}</div>`;
  }
  rows += `</div>`;
  return {
    file: `shipped-${surface}.html`,
    surface,
    shots: [{ out: `gallery/shipped-${surface}.png`, full: true }],
    html: page({
      title: `awl app icons — shipped — ${surface}`,
      surface,
      body: `<div class="sheet"><h1>awl app icons — the shipped set</h1>
<p class="sub">Each world at the ONE logo-cursor its world literal assigns (<code>Theme::icon_cursor</code>), down the sizes a Dock and an app switcher actually draw. ${surface} surface.</p>
<div class="strip">${rows}</div></div>`,
    }),
  };
}

/** The shipped assignment for one world, bounded for the review dashboard. */
function shippedWorldPage(manifest, tuning, world, faceOf) {
  const shown = [256, 128, 64, 44, 32, 24];
  if (!tuning.presets[world.cursor]) {
    throw new Error(`${world.name} declares cursor ${world.cursor}, which is not one of the three presets`);
  }
  const geom = geometry(tuning, world.cursor, world.font);
  const block = (surface) => {
    const s = SURFACES[surface];
    let row = `<div class="grid" style="grid-template-columns:repeat(${shown.length},auto);justify-content:start">`;
    row += shown.map((size) => `<div class="hdr">${size}px</div>`).join("");
    row += shown.map((size) => `<div>${tile(world, faceOf(world.font), geom, size)}</div>`).join("");
    row += `</div>`;
    return `<div style="background:${s.page};color:${s.text};padding:24px 28px">
<p class="cap" style="color:${s.dim};margin:0 0 16px">${surface} Dock surface</p>
<div class="strip" style="background:${s.strip}">${row}</div></div>`;
  };
  return {
    file: `shipped-world-${world.name}.html`,
    surface: "dark",
    shots: [{ out: `gallery/shipped-world-${world.name}.png`, full: true }],
    html: page({
      title: `awl app icon — shipped — ${world.name}`,
      surface: "dark",
      body: `<div style="padding:32px 32px 0"><h1>${esc(world.name)} — shipped icon</h1>
<p class="sub">${esc(world.font)} &middot; ${esc(
        tuning.presets[world.cursor].label
      )} from <code>Theme::icon_cursor</code> &middot; native size ladder</p></div>
${block("dark")}${block("light")}`,
    }),
  };
}

/** One world, all three presets, big and small, on both surfaces at once. */
function worldPage(manifest, tuning, world, faceOf) {
  const presets = Object.keys(tuning.presets);
  const block = (surface) => {
    const s = SURFACES[surface];
    let rows = `<div class="grid" style="grid-template-columns: 150px repeat(${SIZE_ROW.length}, auto); justify-content:start">`;
    rows += `<div></div>` + SIZE_ROW.map((x) => `<div class="hdr">${x}px</div>`).join("");
    for (const preset of presets) {
      const geom = geometry(tuning, preset, world.font);
      rows += `<div class="world" style="width:150px">${esc(tuning.presets[preset].label)}</div>`;
      for (const size of SIZE_ROW) rows += `<div>${tile(world, faceOf(world.font), geom, size)}</div>`;
    }
    rows += `</div>`;
    return `<div style="background:${s.page};color:${s.text};padding:28px 32px">
<p class="cap" style="color:${s.dim};margin:0 0 16px">${surface} surface</p>
<div class="strip" style="background:${s.strip}">${rows}</div></div>`;
  };
  return {
    file: `world-${world.name}.html`,
    surface: "dark",
    shots: [{ out: `gallery/world-${world.name}.png`, full: true }],
    html: page({
      title: `awl icon — ${world.name}`,
      surface: "dark",
      body: `<div style="padding:32px 32px 0"><h1>${esc(world.name)}</h1>
<p class="sub">${esc(world.font)} &middot; ground ${world.ground} &middot; ink ${world.base_content} &middot; cursor ${world.primary} &middot; cursor ink ${world.primary_content}</p></div>
${block("dark")}${block("light")}`,
    }),
  };
}

// ------------------------------------------------------------------ main ---

function main() {
  const args = process.argv.slice(2);
  const arg = (name, dflt) => {
    const i = args.indexOf(name);
    return i >= 0 ? args[i + 1] : dflt;
  };
  const manifestPath = arg("--manifest");
  const fontsDir = arg("--fonts", "assets/fonts");
  const outDir = arg("--out");
  const tuningPath = arg("--tuning", path.join(path.dirname(new URL(import.meta.url).pathname), "tuning.json"));
  if (!manifestPath || !outDir) {
    console.error("usage: build.mjs --manifest M.json --out DIR [--fonts assets/fonts] [--tuning tuning.json]");
    process.exit(2);
  }
  const manifest = JSON.parse(fs.readFileSync(manifestPath, "utf8"));
  const tuning = JSON.parse(fs.readFileSync(tuningPath, "utf8"));
  if (manifest.schema !== 3) throw new Error(`manifest schema ${manifest.schema} is not the 3 this builder reads`);
  assertNoWorldKeys(tuning, manifest);
  const faceOf = (family) => {
    const f = manifest.faces.find((x) => x.family === family);
    if (!f) throw new Error(`no bundled face for ${family}`);
    return f;
  };

  fs.mkdirSync(outDir, { recursive: true });
  fs.writeFileSync(path.join(outDir, "fonts.css"), fontsCss(manifest, fontsDir));
  fs.copyFileSync(
    path.join(path.dirname(new URL(import.meta.url).pathname), "lockup.css"),
    path.join(outDir, "lockup.css")
  );

  const pages = [];
  for (const size of SIZES) pages.push(tilesPage(manifest, tuning, size, faceOf));
  for (const surface of ["dark", "light"]) pages.push(overviewPage(manifest, tuning, surface, faceOf));
  for (const surface of ["dark", "light"]) pages.push(shippedPage(manifest, tuning, surface, faceOf));
  for (const world of manifest.worlds) pages.push(shippedWorldPage(manifest, tuning, world, faceOf));
  for (const preset of Object.keys(tuning.presets)) {
    for (const surface of ["dark", "light"]) {
      pages.push(sizesPage(manifest, tuning, preset, surface, faceOf));
      pages.push(dockPage(manifest, tuning, preset, surface, faceOf));
    }
  }
  for (const world of manifest.worlds) pages.push(worldPage(manifest, tuning, world, faceOf));

  const index = { pages: [] };
  for (const p of pages) {
    fs.writeFileSync(path.join(outDir, p.file), p.html);
    index.pages.push({
      file: p.file,
      transparent: !!p.transparent,
      viewport: p.viewport ?? { w: 2400, h: 1400 },
      shots: p.shots,
    });
  }
  fs.writeFileSync(path.join(outDir, "index.json"), JSON.stringify(index, null, 2) + "\n");
  const shots = index.pages.reduce((n, p) => n + p.shots.length, 0);
  console.error(`build.mjs: ${pages.length} pages, ${shots} shots -> ${outDir}`);
}

main();
