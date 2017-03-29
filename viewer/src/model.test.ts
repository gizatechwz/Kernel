import { test } from "node:test";
import assert from "node:assert/strict";
import { humanBytes, parseViewerDocument } from "./model.js";

const valid = {
  label: "before",
  backend: "fixture",
  interval_ms: 250,
  t_ms: [0, 250, 500],
  series: [
    {
      pid: 5000,
      ppid: 4200,
      comm: "cargo",
      cpu_pct: [0, 12.5, 8.0],
      rss_bytes: [12582912, 20971520, 22020096],
    },
  ],
};

test("parses a well-formed viewer document", () => {
  const doc = parseViewerDocument(valid);
  assert.equal(doc.label, "before");
  assert.equal(doc.series.length, 1);
  assert.equal(doc.series[0].comm, "cargo");
  assert.deepEqual(doc.t_ms, [0, 250, 500]);
});

test("rejects misaligned series arrays", () => {
  const bad = structuredClone(valid);
  bad.series[0].cpu_pct = [1, 2]; // shorter than t_ms
  assert.throws(() => parseViewerDocument(bad), /must match 3 frames/);
