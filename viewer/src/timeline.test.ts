import { test } from "node:test";
import assert from "node:assert/strict";
import { parseViewerDocument } from "./model.js";
import { renderStandaloneHtml, renderTimelineSvg } from "./timeline.js";

const doc = parseViewerDocument({
  label: "after",
  backend: "fixture",
  interval_ms: 250,
  t_ms: [0, 250, 500, 750],
  series: [
    {
      pid: 6000,
      ppid: 4200,
      comm: "cargo",
      cpu_pct: [0, 20, 16, 12],
      rss_bytes: [12582912, 18874368, 20971520, 22020096],
    },
    {
      pid: 6010,
      ppid: 6000,
      comm: "rustc",
      cpu_pct: [0, 80, 120, 40],
      rss_bytes: [0, 75497472, 234881024, 318767104],
    },
  ],
});

test("renders valid SVG root with a viewBox", () => {
  const svg = renderTimelineSvg(doc);
  assert.match(svg, /^<svg xmlns="http:\/\/www\.w3\.org\/2000\/svg"/);
  assert.match(svg, /viewBox="0 0 \d+ \d+"/);
  assert.ok(svg.trim().endsWith("</svg>"));
});

test("includes one polyline per series with data", () => {
  const svg = renderTimelineSvg(doc);
  const polylines = svg.match(/<polyline /g) ?? [];
  assert.equal(polylines.length, 2);
});

test("labels every process by pid and comm", () => {
  const svg = renderTimelineSvg(doc);
  assert.match(svg, /6000 cargo/);
  assert.match(svg, /6010 rustc/);
});

test("escapes XML-sensitive characters in comm", () => {
  const tricky = parseViewerDocument({
    label: "x",
    backend: "fixture",
    interval_ms: 100,
