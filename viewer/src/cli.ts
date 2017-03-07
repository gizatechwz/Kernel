#!/usr/bin/env node
/**
 * Node renderer: reads a kernelkite viewer document (JSON) and writes an SVG or
 * standalone HTML file. No remote dependencies.
 *
 * Usage:
 *   kernelkite-render <viewer.json> [out.svg|out.html]
 *
 * The output format is chosen from the extension of the output path (.html for
 * a full page, otherwise SVG). If no output path is given, SVG is written to
 * stdout.
 */

import { readFileSync, writeFileSync } from "node:fs";
import { parseViewerDocument } from "./model.js";
import { renderStandaloneHtml, renderTimelineSvg } from "./timeline.js";

function main(argv: string[]): number {
  const args = argv.slice(2);
  if (args.length === 0 || args[0] === "-h" || args[0] === "--help") {
    process.stderr.write(
      "usage: kernelkite-render <viewer.json> [out.svg|out.html]\n",
    );
    return args.length === 0 ? 2 : 0;
  }

  const inputPath = args[0];
  const outPath = args[1];

  let raw: string;
  try {
    raw = readFileSync(inputPath, "utf8");
  } catch (e) {
    process.stderr.write(`error: cannot read ${inputPath}: ${String(e)}\n`);
    return 1;
  }

  let doc;
  try {
    doc = parseViewerDocument(JSON.parse(raw));
  } catch (e) {
    process.stderr.write(`error: invalid viewer document: ${String(e)}\n`);
    return 1;
  }

  const svg = renderTimelineSvg(doc);

  if (!outPath) {
    process.stdout.write(svg + "\n");
    return 0;
  }
