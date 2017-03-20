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
