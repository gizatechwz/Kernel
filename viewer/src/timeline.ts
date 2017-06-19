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
  return points.join(" ");
}

/**
 * Render a full timeline SVG string. `maxCpu` normalises every lane to the same
 * scale so lanes are visually comparable; it defaults to the global maximum.
 */
export function renderTimelineSvg(
  doc: ViewerDocument,
  options: Partial<TimelineOptions> = {},
): string {
  const opt = { ...DEFAULT_OPTIONS, ...options };
  const lanes = doc.series.length;
  const plotX = opt.labelWidth + opt.padding;
  const plotWidth = opt.width - plotX - opt.padding;
  const headerHeight = 52;
  const height = headerHeight + lanes * opt.laneHeight + opt.padding;

  const globalMaxCpu = Math.max(
    1,
    ...doc.series.flatMap((s) => s.cpu_pct),
  );

  const parts: string[] = [];
  parts.push(
    `<svg xmlns="http://www.w3.org/2000/svg" width="${opt.width}" height="${height}" ` +
      `viewBox="0 0 ${opt.width} ${height}" role="img" ` +
      `aria-label="kernelkite CPU timeline for ${escapeXml(doc.label)}">`,
  );
  parts.push(
    `<rect x="0" y="0" width="${opt.width}" height="${height}" fill="#0d1117"/>`,
  );
  parts.push(
    `<text x="${opt.padding}" y="24" fill="#e6edf3" font-family="monospace" ` +
      `font-size="16" font-weight="bold">kernelkite — ${escapeXml(doc.label)} ` +
      `[${escapeXml(doc.backend)}]</text>`,
  );
  const durMs = doc.t_ms.length > 0 ? doc.t_ms[doc.t_ms.length - 1] : 0;
  parts.push(
    `<text x="${opt.padding}" y="42" fill="#8b949e" font-family="monospace" ` +
      `font-size="11">${doc.series.length} processes · ${doc.t_ms.length} frames · ` +
      `${durMs} ms · interval ${doc.interval_ms} ms</text>`,
  );

  doc.series.forEach((s, i) => {
    const laneTop = headerHeight + i * opt.laneHeight;
    const colour = PALETTE[i % PALETTE.length];
    // Lane separator.
    parts.push(
      `<line x1="${opt.padding}" y1="${laneTop}" x2="${opt.width - opt.padding}" ` +
        `y2="${laneTop}" stroke="#21262d" stroke-width="1"/>`,
    );
    // Label: pid + comm.
    parts.push(
      `<text x="${opt.padding}" y="${laneTop + opt.laneHeight / 2 + 4}" fill="${colour}" ` +
        `font-family="monospace" font-size="12">${s.pid} ${escapeXml(s.comm)}</text>`,
    );
    // Peak RSS annotation on the right of the label column.
    const peakRss = Math.max(0, ...s.rss_bytes);
    parts.push(
      `<text x="${opt.labelWidth}" y="${laneTop + opt.laneHeight / 2 + 4}" fill="#8b949e" ` +
        `font-family="monospace" font-size="10" text-anchor="end">` +
        `${humanBytes(peakRss)}</text>`,
    );
    // CPU polyline.
    const pts = cpuPolyline(
      s,
      plotX,
      plotWidth,
      laneTop,
      opt.laneHeight,
      globalMaxCpu,
