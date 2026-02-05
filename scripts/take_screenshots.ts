#!/usr/bin/env bun
//
// Captures screenshots of Ratado running in Terminal.app (macOS only).
//
// Prerequisites:
//   - cargo build --release --examples
//   - bun, ImageMagick (magick), Screen Recording permission for Terminal
//
// Usage:
//   bun scripts/take_screenshots.ts

import { $ } from "bun";
import path from "path";

const ROOT = path.resolve(import.meta.dir, "..");
const BINARY = `${ROOT}/target/release/ratado`;
const SEED_BINARY = `${ROOT}/target/release/examples/seed_demo`;
const DB = `${ROOT}/target/demo.db`;
const DOCS = `${ROOT}/docs`;

// Re-seed the database
await $`${SEED_BINARY} ${DB}`.quiet();

const sleep = (ms: number) => new Promise((r) => setTimeout(r, ms));

async function captureTerminalWindow(outputFile: string) {
  const winId = (
    await $`osascript -e 'tell application "Terminal" to return id of front window'`.text()
  ).trim();
  await $`screencapture -x -o -l ${winId} ${outputFile}`.quiet();
}

async function sendKeystroke(key: string) {
  await $`osascript -e '
    tell application "Terminal" to activate
    delay 0.2
    tell application "System Events"
      keystroke "${key}"
    end tell
  '`.quiet();
}

// Step 1: Open Terminal
await $`osascript -e '
  tell application "Terminal"
    activate
    do script "${BINARY} -d ${DB}"
    delay 0.3
    set bounds of front window to {0, 0, 1600, 1000}
  end tell
'`;

// Step 2: Capture splash
await sleep(1200);
await captureTerminalWindow(`${DOCS}/screenshot_splash_raw.png`);
console.log("Captured: splash");

// Step 3: Capture tasks
await sleep(6000);
await captureTerminalWindow(`${DOCS}/screenshot_tasks_raw.png`);
console.log("Captured: tasks");

// Step 4: Calendar
await sendKeystroke("c");
await sleep(1500);
await captureTerminalWindow(`${DOCS}/screenshot_calendar_raw.png`);
console.log("Captured: calendar");

// Step 5: Quit
await sleep(300);
await sendKeystroke("q");

// Step 6: Precise crop + resize
// Measured at 2x Retina on 3200x2000 raw:
//   Left border:   20px white (dark starts at x=20)
//   Right border:  30px white (dark ends at x=3170)
//   Top border:    78px white+titlebar (dark starts at y=78)
//   Bottom border: 46px white (dark ends at y=1954)
// Crop region: x=20, y=78, w=3150, h=1876
const CROP_X = 20;
const CROP_Y = 78;
const CROP_W = 3150;
const CROP_H = 1876;

for (const name of ["splash", "tasks", "calendar"]) {
  const raw = `${DOCS}/screenshot_${name}_raw.png`;
  const out = `${DOCS}/screenshot_${name}.png`;

  // Crop precisely at full resolution, then resize 50% for 1x
  await $`magick ${raw} -crop ${CROP_W}x${CROP_H}+${CROP_X}+${CROP_Y} +repage -resize 50% -strip ${out}`.quiet();
  await $`rm ${raw}`.quiet();

  const info = (await $`magick identify -format '%wx%h' ${out}`.text()).trim();
  const size = (await $`wc -c < ${out}`.text()).trim();
  const kb = Math.round(parseInt(size) / 1024);
  console.log(`  ${name}: ${info} (${kb}K)`);
}

console.log("\nDone!");
