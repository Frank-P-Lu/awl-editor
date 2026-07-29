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
//
// NO SILENT STALL: every wait in here is named and bounded. A CDP call that
// never answers, a browser that dies mid-run, a page that never fires load —
// each ends as one line naming the stage, not as a process that sits forever
// with nothing to say. `render-laws.mjs` holds those guards to their word.

import { spawn } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

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

// One budget for every wait. Overridable per run (`--timeout-ms`) because the
// honest number depends on the machine, and because setting it to 1 is how the
// stall diagnostics are proven non-vacuous.
export const DEFAULT_TIMEOUT_MS = 60_000;
// How much of the browser's own stderr to keep for the failure report. Bounded:
// a wedged GPU process can log without end, and the last words are the useful
// ones.
const STDERR_TAIL = 8 * 1024;

/** A named stall. Its message says which stage stopped answering, and for how long. */
export class StallError extends Error {
  constructor(stage, ms) {
    super(`render.mjs: STALLED in ${stage} — no answer after ${ms}ms`);
    this.name = "StallError";
    this.stage = stage;
  }
}

/**
 * Await `work`, but never past `ms`. The stage name is the whole point: an
 * exporter that hangs without saying where is a defect on its own terms, since
 * the next person has nothing to bisect.
 */
export async function stage(name, work, ms) {
  let timer;
  try {
    return await Promise.race([
      work,
      new Promise((_, reject) => {
        timer = setTimeout(() => reject(new StallError(name, ms)), ms);
      }),
    ]);
  } finally {
    clearTimeout(timer);
  }
}

/** Poll `probe` until it returns something truthy, or fail by name. */
async function pollFor(name, probe, ms) {
  const deadline = Date.now() + ms;
  for (;;) {
    const got = await probe();
    if (got) return got;
    if (Date.now() >= deadline) throw new StallError(name, ms);
    await sleep(50);
  }
}

/**
 * Kill everything THIS run started and take its scratch profile with it.
 *
 * Idempotent, and safe to call from a signal handler or `process.on("exit")`.
 * The browser is spawned `detached`, so it leads its own process group and one
 * negative-pid signal reaches its helpers too — the exporter never selects a
 * process by NAME, so a browser some other tool is running is never at risk.
 */
export function shutdown(browser) {
  if (!browser || browser.down) return;
  browser.down = true;
  try {
    browser.cdp?.close();
  } catch {}
  try {
    process.kill(-browser.proc.pid, "SIGKILL");
  } catch {}
  try {
    browser.proc.kill("SIGKILL");
  } catch {}
  // The SIGKILLed browser can still have writes in flight against its own
  // profile directory, so a bare rmSync races it and throws ENOTEMPTY —
  // failing a render that had already written every PNG. Retry the unlink
  // rather than reporting a successful export as broken.
  fs.rmSync(browser.profile, { recursive: true, force: true, maxRetries: 20, retryDelay: 100 });
}

export async function launch(timeoutMs = DEFAULT_TIMEOUT_MS) {
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
    detached: true,
  });
  const browser = { proc, profile, port: null, cdp: null, down: false, stderr: "", exit: null };

  // THE PIPE MUST BE READ. `stdio: "pipe"` with no consumer is a deadlock
  // waiting for a noisy run: once the OS pipe buffer fills, the browser's next
  // write to stderr blocks inside the browser, and every CDP request in flight
  // — `Page.captureScreenshot` included — stops being answered, forever, with
  // the explanation stuck unread in the pipe. Draining costs nothing and turns
  // the browser's own words into the failure report. (`render-laws.mjs` law 1.)
  proc.stderr.setEncoding("utf8");
  proc.stderr.on("data", (chunk) => {
    browser.stderr = (browser.stderr + chunk).slice(-STDERR_TAIL);
  });
  proc.on("exit", (code, signal) => {
    browser.exit = signal ? `signal ${signal}` : `code ${code}`;
  });

  const portFile = path.join(profile, "DevToolsActivePort");
  try {
    browser.port = await pollFor(
      "launch/devtools-port",
      () => {
        if (browser.exit) throw new Error(`render.mjs: browser exited during launch (${browser.exit})`);
        if (!fs.existsSync(portFile)) return null;
        return fs.readFileSync(portFile, "utf8").split("\n")[0].trim() || null;
      },
      timeoutMs
    );
  } catch (e) {
    // The old code left the scratch profile behind on this path; thirteen of
    // them had accumulated in $TMPDIR by the time anyone looked.
    shutdown(browser);
    throw e;
  }
  return browser;
}

/** A minimal CDP session over the page target's WebSocket. */
export async function connect(browser, timeoutMs = DEFAULT_TIMEOUT_MS) {
  const target = await pollFor(
    "connect/page-target",
    async () => {
      if (browser.exit) throw new Error(`render.mjs: browser exited before it served a page target (${browser.exit})`);
      const res = await fetch(`http://127.0.0.1:${browser.port}/json/list`).catch(() => null);
      if (!res) return null;
      return (await res.json()).find((t) => t.type === "page") ?? null;
    },
    timeoutMs
  );
  const ws = new WebSocket(target.webSocketDebuggerUrl);
  await stage(
    "connect/websocket",
    new Promise((res, rej) => {
      ws.addEventListener("open", res, { once: true });
      ws.addEventListener("error", rej, { once: true });
    }),
    timeoutMs
  );
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
  // A browser that dies mid-run used to be indistinguishable from a slow one:
  // its half of the conversation simply stopped. Fail everything outstanding
  // the moment the socket or the process goes, and say which.
  const abort = (why) => {
    const err = new Error(`render.mjs: ${why}`);
    for (const [, { reject }] of pending) reject(err);
    pending.clear();
    for (const [, waiters] of events) for (const w of waiters) w.reject?.(err);
    events.clear();
  };
  ws.addEventListener("close", () => abort("the DevTools connection closed mid-run"), { once: true });
  browser.proc.on("exit", () => abort(`the browser exited mid-run (${browser.exit})`));
  // `abort` may reject a promise whose only reader already gave up — a stage
  // that timed out, or a `once()` armed for a page the run never reached. An
  // inert handler keeps that from surfacing as an unhandled rejection, which
  // would replace the named stall with a crash about the wrong thing.
  const inert = (p) => (p.catch(() => {}), p);
  return {
    send(method, params = {}) {
      id += 1;
      const mid = id;
      return inert(
        new Promise((resolve, reject) => {
          pending.set(mid, { resolve, reject });
          ws.send(JSON.stringify({ id: mid, method, params }));
        })
      );
    },
    once(method) {
      return inert(
        new Promise((resolve, reject) => {
          if (!events.has(method)) events.set(method, []);
          const waiter = (params) => resolve(params);
          waiter.reject = reject;
          events.get(method).push(waiter);
        })
      );
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
  const timeoutMs = Number(arg("--timeout-ms", DEFAULT_TIMEOUT_MS));
  const index = JSON.parse(fs.readFileSync(path.join(buildDir, "index.json"), "utf8"));

  const browser = await launch(timeoutMs);
  // From here on the teardown owns the browser no matter how the run ends —
  // including an exception out of `connect`, which used to escape the `finally`
  // below and leave a headless Chromium running with nobody's name on it.
  const bye = () => shutdown(browser);
  process.on("exit", bye);
  for (const sig of ["SIGINT", "SIGTERM", "SIGHUP"]) process.on(sig, () => process.exit(130));
  let written = 0;
  try {
    const cdp = await connect(browser, timeoutMs);
    browser.cdp = cdp;
    const at = (name, work) => stage(name, work, timeoutMs);
    await at("Page.enable", cdp.send("Page.enable"));
    for (const page of index.pages) {
      if (only && !page.file.includes(only)) continue;
      await at(
        `${page.file}: setDeviceMetricsOverride`,
        cdp.send("Emulation.setDeviceMetricsOverride", {
          width: page.viewport.w,
          height: page.viewport.h,
          deviceScaleFactor: 1,
          mobile: false,
        })
      );
      // A transparent default background is what gives the tiles their real
      // macOS shape: everything outside the squircle is alpha 0, not "the
      // colour of whatever page it happened to be composed on".
      await at(
        `${page.file}: setDefaultBackgroundColorOverride`,
        cdp.send("Emulation.setDefaultBackgroundColorOverride", {
          color: page.transparent ? { r: 0, g: 0, b: 0, a: 0 } : { r: 255, g: 255, b: 255, a: 1 },
        })
      );
      const loaded = cdp.once("Page.loadEventFired");
      await at(`${page.file}: Page.navigate`, cdp.send("Page.navigate", { url: `file://${path.join(buildDir, page.file)}` }));
      await at(`${page.file}: load event`, loaded);
      // Fonts are data: URLs, so this settles immediately — but WAITING on it
      // is what makes the render deterministic instead of a race with layout.
      const ready = await at(
        `${page.file}: fonts.ready`,
        cdp.send("Runtime.evaluate", {
          expression: "document.fonts.ready.then(() => document.fonts.status)",
          awaitPromise: true,
          returnByValue: true,
        })
      );
      if (ready.result.value !== "loaded") throw new Error(`${page.file}: fonts did not load`);

      // THE VIEWPORT IS GROWN TO FIT THE WHOLE DOCUMENT, plus a margin.
      // Determinism depends on it: with the viewport ending exactly where the
      // content does, tiles in the last column straddle the raster edge and
      // their border-radius antialiasing came out ±3/255 different between two
      // runs of the same page (caught by --check; three of 572 PNGs). Inside a
      // viewport that fully contains the document, every tile rasterizes the
      // same way every time and `captureBeyondViewport` is never needed.
      const dims = await at(
        `${page.file}: measure .page`,
        cdp.send("Runtime.evaluate", {
          expression:
            "(() => { const e = document.querySelector('.page'); const r = e.getBoundingClientRect();" +
            " return JSON.stringify([Math.ceil(r.right), Math.ceil(r.bottom)]); })()",
          returnByValue: true,
        })
      );
      const [docW, docH] = JSON.parse(dims.result.value);
      const viewW = Math.min(docW + MARGIN, MAX_VIEWPORT);
      const viewH = Math.min(docH + MARGIN, MAX_VIEWPORT);
      const beyond = viewW < docW || viewH < docH;
      await at(
        `${page.file}: grow viewport to ${viewW}x${viewH}`,
        cdp.send("Emulation.setDeviceMetricsOverride", {
          width: viewW,
          height: viewH,
          deviceScaleFactor: 1,
          mobile: false,
        })
      );

      for (const shot of page.shots) {
        const clip = shot.full
          ? { x: 0, y: 0, width: docW, height: docH, scale: 1 }
          : { x: shot.x, y: shot.y, width: shot.w, height: shot.h, scale: 1 };
        for (const v of Object.values(clip)) {
          if (!Number.isInteger(v)) throw new Error(`${shot.out}: non-integer clip ${JSON.stringify(clip)}`);
        }
        const res = await at(
          `capture ${shot.out}`,
          cdp.send("Page.captureScreenshot", {
            format: "png",
            clip,
            captureBeyondViewport: beyond,
            fromSurface: true,
            optimizeForSpeed: false,
          })
        );
        const dest = path.join(outDir, shot.out);
        fs.mkdirSync(path.dirname(dest), { recursive: true });
        fs.writeFileSync(dest, Buffer.from(res.data, "base64"));
        written += 1;
      }
    }
  } catch (e) {
    // The browser's last words are the diagnosis when a stage stalls, and they
    // are only readable because the stderr pipe is drained.
    const tail = browser.stderr.trim();
    if (tail) console.error(`render.mjs: browser stderr (last ${STDERR_TAIL}B):\n${tail}`);
    if (browser.exit) console.error(`render.mjs: browser had exited (${browser.exit})`);
    console.error(`render.mjs: ${written} PNGs were written before the failure`);
    throw e;
  } finally {
    shutdown(browser);
    process.off("exit", bye);
  }
  console.error(`render.mjs: wrote ${written} PNGs -> ${outDir}`);
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  main().catch((e) => {
    console.error(String(e.stack ?? e));
    process.exit(1);
  });
}
