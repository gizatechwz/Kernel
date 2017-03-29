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
