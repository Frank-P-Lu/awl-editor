#!/usr/bin/env node
// THE RENDERER'S LAWS — the four guards that stand between this pipeline and
// the failure it actually had: a Chromium that stopped answering
// `Page.captureScreenshot`, no message, no exit, no cleanup, thirteen scratch
// profiles left in $TMPDIR and a browser nobody could account for.
//
//   node scripts/icons/render-laws.mjs
//
// These drive the SHIPPED functions out of `render.mjs` — not copies — so a
// guard that is deleted or weakened takes a law with it. `export-icons.sh` runs
// them before it renders anything, because a five-second proof is cheaper than
// a hang at 03:00.
//
// No test framework, no dependency: same rule as the renderer itself.

import { spawn } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";

import { connect, launch, shutdown, stage, StallError } from "./render.mjs";

const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
const profileGlob = () => fs.readdirSync(os.tmpdir()).filter((n) => n.startsWith("awl-icons-"));

let failed = 0;
async function law(name, body) {
  try {
    await body();
    console.log(`  ok   ${name}`);
  } catch (e) {
    failed += 1;
    console.log(`  FAIL ${name}\n       ${String(e.message ?? e).replace(/\n/g, "\n       ")}`);
  }
}
const assert = (cond, msg) => {
  if (!cond) throw new Error(msg);
};

console.log("render-laws.mjs");

// 1a. THE HAZARD IS REAL. `stdio: [_, _, "pipe"]` with nobody reading is not a
// tidiness question: the child blocks in write(2) as soon as the OS pipe buffer
// fills, and a blocked browser answers no CDP request ever again. This law
// fails if that stops being true — at which point law 1b's drain is free to go.
await law("an unread stderr pipe blocks the child (the deadlock the drain prevents)", async () => {
  const marker = path.join(os.tmpdir(), `awl-law-pipe-${process.pid}`);
  const chatter = ["-c", `i=0; while [ $i -lt 4096 ]; do printf '%01024d' 0 >&2; i=$((i+1)); done; : > ${marker}`];
  for (const drained of [false, true]) {
    fs.rmSync(marker, { force: true });
    const proc = spawn("/bin/sh", chatter, { stdio: ["ignore", "ignore", "pipe"] });
    if (drained) proc.stderr.resume();
    await sleep(4000);
    const finished = fs.existsSync(marker);
    proc.kill("SIGKILL");
    fs.rmSync(marker, { force: true });
    assert(finished === drained, `4MB of stderr, drained=${drained}: child finished=${finished}, expected ${drained}`);
  }
});

// 1b. AND THE DRAIN IS LIVE. The browser's own words are the diagnosis when a
// stage stalls; they only exist because something is reading the pipe.
await law("launch() reads the browser's stderr", async () => {
  const browser = await launch();
  try {
    await sleep(500);
    assert(browser.stderr.length > 0, `nothing was read from the browser's stderr (got ${browser.stderr.length} bytes)`);
  } finally {
    shutdown(browser);
  }
});

// 2. THE NAMED STALL — on the exact call that hung. A renderer spinning on its
// main thread never produces the frame `fromSurface` capture waits for, so this
// is the real hang, reproduced, not a mocked timer.
await law("a capture that never answers fails by name", async () => {
  const browser = await launch();
  const page = path.join(os.tmpdir(), `awl-law-page-${process.pid}.html`);
  fs.writeFileSync(page, "<html><body style='margin:0'><div style='width:160px;height:160px;background:#c33'></div></body></html>");
  try {
    const cdp = await connect(browser);
    browser.cdp = cdp;
    await cdp.send("Page.enable");
    await cdp.send("Emulation.setDeviceMetricsOverride", { width: 200, height: 200, deviceScaleFactor: 1, mobile: false });
    const loaded = cdp.once("Page.loadEventFired");
    await cdp.send("Page.navigate", { url: `file://${page}` });
    await loaded;
    cdp.send("Runtime.evaluate", { expression: "const t = Date.now(); while (Date.now() - t < 600000) {}" });
    await sleep(500);
    const shot = "tiles/Tawny-block-32.png";
    let caught = null;
    try {
      await stage(
        `capture ${shot}`,
        cdp.send("Page.captureScreenshot", {
          format: "png",
          clip: { x: 0, y: 0, width: 160, height: 160, scale: 1 },
          captureBeyondViewport: false,
          fromSurface: true,
          optimizeForSpeed: false,
        }),
        6000
      );
    } catch (e) {
      caught = e;
    }
    assert(caught instanceof StallError, `a wedged renderer's capture did not raise StallError (got ${caught})`);
    assert(caught.message.includes(shot), `the stall does not name the shot: ${caught.message}`);
    assert(caught.stage.startsWith("capture "), `the stall does not name the stage: ${caught.stage}`);
  } finally {
    shutdown(browser);
    fs.rmSync(page, { force: true });
  }
});

// 3. NOTHING SURVIVES THE RUN. The browser leads its own process group, so one
// signal reaches its helpers — and the exporter never picks a process by name,
// so a browser some other tool is running is never collateral.
await law("shutdown() leaves no process and no profile behind", async () => {
  const browser = await launch();
  const { pid } = browser.proc;
  const profile = browser.profile;
  const helpers = () => {
    try {
      process.kill(-pid, 0);
      return true;
    } catch {
      return false;
    }
  };
  assert(helpers(), "the browser's process group was gone before shutdown");
  shutdown(browser);
  await sleep(1000);
  assert(!helpers(), `the process group of pid ${pid} outlived shutdown()`);
  assert(!fs.existsSync(profile), `the scratch profile ${profile} outlived shutdown()`);
});

// 4. INCLUDING THE RUNS THAT NEVER GOT STARTED. The launch failure path used to
// return through a `throw` that skipped every cleanup, which is why $TMPDIR had
// collected thirteen abandoned profiles.
await law("a launch that times out leaves no scratch profile", async () => {
  const before = new Set(profileGlob());
  let caught = null;
  try {
    await launch(1);
  } catch (e) {
    caught = e;
  }
  assert(caught instanceof StallError, `a 1ms launch budget did not raise StallError (got ${caught})`);
  assert(caught.stage === "launch/devtools-port", `the stall does not name the launch stage: ${caught.stage}`);
  const leaked = profileGlob().filter((n) => !before.has(n));
  assert(leaked.length === 0, `the failed launch leaked ${leaked.length} profile(s): ${leaked.join(", ")}`);
});

console.log(failed === 0 ? "render-laws.mjs: all laws hold" : `render-laws.mjs: ${failed} law(s) FAILED`);
process.exit(failed === 0 ? 0 : 1);
