#!/usr/bin/env node
// THE RENDERER — drives ONE pinned, offline Chromium over the DevTools
// protocol and writes every PNG the builder's index asks for.
//
// PINNED: the browser is a fixed local revision (CHROMIUM_REVISION below,
// overridable with AWL_ICON_CHROMIUM for a different local checkout). Nothing
// is downloaded — if the revision is not already on the machine the script
// stops and says so. There is no npm dependency either: Node's own global
// fetch + WebSocket speak CDP directly, so this file has no lockfile, no
// node_modules, and no supply chain.
//
// OFFLINE: the browser is launched with the network stack disabled at the
// command line, and the pages it loads inline their fonts as data: URLs. An
// icon pipeline that could reach the network would be a hole in the same
// zero-network invariant the editor itself keeps.
//
// DETERMINISTIC: fixed device scale, sRGB forced, LCD subpixel text off (icons
// carry alpha; subpixel AA would bake the surface color into the glyph edges),
// GPU rasterization off. Same pages in, byte-identical PNGs out.

import { spawn } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";

// Chrome tops out at 16384px; the biggest page here (54 tiles at 1024) is
// ~6.3k x 9.4k, so every page fits and nothing is ever captured beyond the
// viewport in practice.
const MAX_VIEWPORT = 16000;
const MARGIN = 64;

const CHROMIUM_REVISION = "chromium-1228";
const CHROMIUM_APP = "chrome-mac-arm64/Google Chrome for Testing.app/Contents/MacOS/Google Chrome for Testing";

function chromiumPath() {
  const override = process.env.AWL_ICON_CHROMIUM;
  if (override) return override;
  return path.join(os.homedir(), "Library/Caches/ms-playwright", CHROMIUM_REVISION, CHROMIUM_APP);
}

const FLAGS = [
  "--headless=new",
  "--disable-gpu",
  "--hide-scrollbars",
  "--force-device-scale-factor=1",
  "--force-color-profile=srgb",
  "--disable-lcd-text",
  "--disable-font-subpixel-positioning",
  "--font-render-hinting=none",
  "--no-first-run",
  "--no-default-browser-check",
  "--disable-extensions",
  "--disable-background-networking",
  "--disable-component-update",
  "--disable-default-apps",
  "--disable-sync",
  "--no-pings",
  "--mute-audio",
  "--metrics-recording-only",
  "--disable-features=Translate,OptimizationHints,MediaRouter,BackForwardCache",
];

const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

async function launch() {
  const bin = chromiumPath();
  if (!fs.existsSync(bin)) {
    throw new Error(
      `pinned Chromium ${CHROMIUM_REVISION} not found at ${bin}\n` +
        `This script never downloads a browser. Install that revision locally, or point AWL_ICON_CHROMIUM at one.`
    );
  }
  const profile = fs.mkdtempSync(path.join(os.tmpdir(), "awl-icons-"));
  const proc = spawn(bin, [...FLAGS, `--user-data-dir=${profile}`, "--remote-debugging-port=0", "about:blank"], {
    stdio: ["ignore", "ignore", "pipe"],
  });
  const portFile = path.join(profile, "DevToolsActivePort");
  for (let i = 0; i < 200; i += 1) {
    if (fs.existsSync(portFile)) {
      const port = fs.readFileSync(portFile, "utf8").split("\n")[0].trim();
      if (port) return { proc, profile, port };
    }
    await sleep(50);
  }
  proc.kill("SIGKILL");
  throw new Error("Chromium never reported a DevTools port");
}

/** A minimal CDP session over the page target's WebSocket. */
async function connect(port) {
  let list = [];
  for (let i = 0; i < 100; i += 1) {
    const res = await fetch(`http://127.0.0.1:${port}/json/list`).catch(() => null);
    if (res) {
      list = await res.json();
      if (list.some((t) => t.type === "page")) break;
    }
    await sleep(50);
  }
  const target = list.find((t) => t.type === "page");
  if (!target) throw new Error("no page target");
  const ws = new WebSocket(target.webSocketDebuggerUrl);
  await new Promise((res, rej) => {
    ws.addEventListener("open", res, { once: true });
    ws.addEventListener("error", rej, { once: true });
  });
  let id = 0;
  const pending = new Map();
  const events = new Map();
  ws.addEventListener("message", (ev) => {
    const msg = JSON.parse(ev.data);
    if (msg.id && pending.has(msg.id)) {
      const { resolve, reject } = pending.get(msg.id);
      pending.delete(msg.id);
      msg.error ? reject(new Error(`${msg.error.message} (${JSON.stringify(msg.error.data ?? "")})`)) : resolve(msg.result);
    } else if (msg.method && events.has(msg.method)) {
      const waiters = events.get(msg.method);
      events.delete(msg.method);
      for (const w of waiters) w(msg.params);
    }
  });
  return {
    send(method, params = {}) {
      id += 1;
      const mid = id;
      return new Promise((resolve, reject) => {
        pending.set(mid, { resolve, reject });
        ws.send(JSON.stringify({ id: mid, method, params }));
      });
    },
    once(method) {
      return new Promise((resolve) => {
        if (!events.has(method)) events.set(method, []);
        events.get(method).push(resolve);
      });
    },
    close: () => ws.close(),
  };
}

async function main() {
  const args = process.argv.slice(2);
  const arg = (n, d) => {
    const i = args.indexOf(n);
    return i >= 0 ? args[i + 1] : d;
  };
  const buildDir = path.resolve(arg("--build"));
  const outDir = path.resolve(arg("--out"));
  const only = arg("--only"); // substring filter over page file names (dev loop)
  const index = JSON.parse(fs.readFileSync(path.join(buildDir, "index.json"), "utf8"));

  const { proc, profile, port } = await launch();
  const cdp = await connect(port);
  let written = 0;
  try {
    await cdp.send("Page.enable");
    for (const page of index.pages) {
      if (only && !page.file.includes(only)) continue;
      await cdp.send("Emulation.setDeviceMetricsOverride", {
        width: page.viewport.w,
        height: page.viewport.h,
        deviceScaleFactor: 1,
        mobile: false,
      });
      // A transparent default background is what gives the tiles their real
      // macOS shape: everything outside the squircle is alpha 0, not "the
      // colour of whatever page it happened to be composed on".
      await cdp.send("Emulation.setDefaultBackgroundColorOverride", {
        color: page.transparent ? { r: 0, g: 0, b: 0, a: 0 } : { r: 255, g: 255, b: 255, a: 1 },
      });
      const loaded = cdp.once("Page.loadEventFired");
      await cdp.send("Page.navigate", { url: `file://${path.join(buildDir, page.file)}` });
      await loaded;
      // Fonts are data: URLs, so this settles immediately — but WAITING on it
      // is what makes the render deterministic instead of a race with layout.
      const ready = await cdp.send("Runtime.evaluate", {
        expression: "document.fonts.ready.then(() => document.fonts.status)",
        awaitPromise: true,
        returnByValue: true,
      });
      if (ready.result.value !== "loaded") throw new Error(`${page.file}: fonts did not load`);

      // THE VIEWPORT IS GROWN TO FIT THE WHOLE DOCUMENT, plus a margin.
      // Determinism depends on it: with the viewport ending exactly where the
      // content does, tiles in the last column straddle the raster edge and
      // their border-radius antialiasing came out ±3/255 different between two
      // runs of the same page (caught by --check; three of 572 PNGs). Inside a
      // viewport that fully contains the document, every tile rasterizes the
      // same way every time and `captureBeyondViewport` is never needed.
      const dims = await cdp.send("Runtime.evaluate", {
        expression:
          "(() => { const e = document.querySelector('.page'); const r = e.getBoundingClientRect();" +
          " return JSON.stringify([Math.ceil(r.right), Math.ceil(r.bottom)]); })()",
        returnByValue: true,
      });
      const [docW, docH] = JSON.parse(dims.result.value);
      const viewW = Math.min(docW + MARGIN, MAX_VIEWPORT);
      const viewH = Math.min(docH + MARGIN, MAX_VIEWPORT);
      const beyond = viewW < docW || viewH < docH;
      await cdp.send("Emulation.setDeviceMetricsOverride", {
        width: viewW,
        height: viewH,
        deviceScaleFactor: 1,
        mobile: false,
      });

      for (const shot of page.shots) {
        const clip = shot.full
          ? { x: 0, y: 0, width: docW, height: docH, scale: 1 }
          : { x: shot.x, y: shot.y, width: shot.w, height: shot.h, scale: 1 };
        for (const v of Object.values(clip)) {
          if (!Number.isInteger(v)) throw new Error(`${shot.out}: non-integer clip ${JSON.stringify(clip)}`);
        }
        const res = await cdp.send("Page.captureScreenshot", {
          format: "png",
          clip,
          captureBeyondViewport: beyond,
          fromSurface: true,
          optimizeForSpeed: false,
        });
        const dest = path.join(outDir, shot.out);
        fs.mkdirSync(path.dirname(dest), { recursive: true });
        fs.writeFileSync(dest, Buffer.from(res.data, "base64"));
        written += 1;
      }
    }
  } finally {
    cdp.close();
    proc.kill("SIGKILL");
    // The SIGKILLed browser can still have writes in flight against its own
    // profile directory, so a bare rmSync races it and throws ENOTEMPTY —
    // failing a render that had already written every PNG. Retry the unlink
    // rather than reporting a successful export as broken.
    fs.rmSync(profile, { recursive: true, force: true, maxRetries: 20, retryDelay: 100 });
  }
  console.error(`render.mjs: wrote ${written} PNGs -> ${outDir}`);
}

main().catch((e) => {
  console.error(String(e.stack ?? e));
  process.exit(1);
});
