/**
 * Types describing the viewer document emitted by `kernelkite viewer`, plus a
 * defensive parser. The document has a shared time axis and one series per
 * process, with CPU% and RSS arrays aligned to that axis.
 */

export interface ViewerSeries {
  pid: number;
  ppid: number;
