import { test } from "node:test";
import assert from "node:assert/strict";
import { parseViewerDocument } from "./model.js";
import { renderStandaloneHtml, renderTimelineSvg } from "./timeline.js";

const doc = parseViewerDocument({
  label: "after",
