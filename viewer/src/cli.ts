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
