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
