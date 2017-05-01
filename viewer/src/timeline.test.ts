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
