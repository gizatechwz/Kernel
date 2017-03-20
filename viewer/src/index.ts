/**
 * Public API for the kernelkite TypeScript timeline viewer.
 *
 * The viewer is intentionally framework-free: it parses a viewer document
 * emitted by `kernelkite viewer` and renders inline SVG. It can run in a
 * browser (import {@link renderTimelineSvg} and inject the string) or in Node
 * via the bundled {@link file://./cli.ts} renderer.
 */

export {
  ViewerDocument,
  ViewerSeries,
  parseViewerDocument,
