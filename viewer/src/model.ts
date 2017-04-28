/**
 * Types describing the viewer document emitted by `kernelkite viewer`, plus a
 * defensive parser. The document has a shared time axis and one series per
 * process, with CPU% and RSS arrays aligned to that axis.
 */

export interface ViewerSeries {
  pid: number;
  ppid: number;
  comm: string;
  /** CPU utilisation percent per frame (can exceed 100 on multi-core). */
  cpu_pct: number[];
  /** Resident memory per frame in bytes. */
  rss_bytes: number[];
}

export interface ViewerDocument {
  label: string;
  backend: string;
  interval_ms: number;
  /** Milliseconds since capture start, one entry per frame. */
  t_ms: number[];
  series: ViewerSeries[];
}

function isNumberArray(x: unknown): x is number[] {
  return Array.isArray(x) && x.every((n) => typeof n === "number" && Number.isFinite(n));
}

/**
 * Parse and validate an unknown value into a {@link ViewerDocument}. Throws a
 * descriptive error if the shape is wrong or the series arrays are not aligned
 * to the time axis — the same invariant the Rust exporter guarantees.
 */
export function parseViewerDocument(input: unknown): ViewerDocument {
  if (typeof input !== "object" || input === null) {
    throw new Error("viewer document must be an object");
  }
  const doc = input as Record<string, unknown>;
  if (typeof doc.label !== "string") throw new Error("missing string field: label");
  if (typeof doc.backend !== "string") throw new Error("missing string field: backend");
  if (typeof doc.interval_ms !== "number") throw new Error("missing number field: interval_ms");
  if (!isNumberArray(doc.t_ms)) throw new Error("t_ms must be an array of numbers");
  if (!Array.isArray(doc.series)) throw new Error("series must be an array");

  const frames = doc.t_ms.length;
  const series: ViewerSeries[] = doc.series.map((raw, i) => {
    if (typeof raw !== "object" || raw === null) {
      throw new Error(`series[${i}] must be an object`);
    }
    const s = raw as Record<string, unknown>;
    if (typeof s.pid !== "number") throw new Error(`series[${i}].pid must be a number`);
    if (typeof s.ppid !== "number") throw new Error(`series[${i}].ppid must be a number`);
    if (typeof s.comm !== "string") throw new Error(`series[${i}].comm must be a string`);
    if (!isNumberArray(s.cpu_pct)) throw new Error(`series[${i}].cpu_pct must be numbers`);
    if (!isNumberArray(s.rss_bytes)) throw new Error(`series[${i}].rss_bytes must be numbers`);
    if (s.cpu_pct.length !== frames || s.rss_bytes.length !== frames) {
      throw new Error(
        `series[${i}] arrays (len ${s.cpu_pct.length}/${s.rss_bytes.length}) ` +
          `must match ${frames} frames`,
      );
    }
    return {
      pid: s.pid,
      ppid: s.ppid,
      comm: s.comm,
      cpu_pct: s.cpu_pct,
      rss_bytes: s.rss_bytes,
    };
  });

  return {
    label: doc.label,
    backend: doc.backend,
