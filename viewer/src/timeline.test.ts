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
