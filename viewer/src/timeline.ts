/**
 * Pure, dependency-free SVG rendering of a {@link ViewerDocument}. The output
 * is a stacked set of per-process CPU sparklines plus a peak-memory bar, all as
 * inline SVG so it works in a browser or as a static file with no remote media.
 */

import { humanBytes, ViewerDocument, ViewerSeries } from "./model.js";

export interface TimelineOptions {
  width: number;
  laneHeight: number;
  labelWidth: number;
  padding: number;
}

export const DEFAULT_OPTIONS: TimelineOptions = {
  width: 900,
  laneHeight: 44,
  labelWidth: 150,
  padding: 16,
};

/** A stable, colour-blind-friendly palette cycled per series. */
const PALETTE = [
  "#4c78a8",
  "#f58518",
  "#54a24b",
  "#e45756",
  "#72b7b2",
  "#b279a2",
  "#ff9da6",
  "#9d755d",
];

function escapeXml(s: string): string {
  return s
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");
}

/** Build an SVG polyline path for one series' CPU% within a lane. */
function cpuPolyline(
  series: ViewerSeries,
  plotX: number,
  plotWidth: number,
  laneTop: number,
  laneHeight: number,
  maxCpu: number,
): string {
  const n = series.cpu_pct.length;
  if (n === 0) return "";
  const denom = maxCpu <= 0 ? 1 : maxCpu;
  const points = series.cpu_pct.map((v, i) => {
    const x = plotX + (n === 1 ? 0 : (i / (n - 1)) * plotWidth);
    const norm = Math.min(v / denom, 1);
    const y = laneTop + laneHeight - 4 - norm * (laneHeight - 8);
    return `${x.toFixed(1)},${y.toFixed(1)}`;
  });
