export function initCanvasSetting(
  canvas: HTMLCanvasElement,
  {
    width,
    height,
    dpr = window.devicePixelRatio,
  }: {
    width: number;
    height: number;
    dpr?: number;
  }
) {
  // Set actual pixel size for canvas
  canvas.width = width * dpr;
  canvas.height = height * dpr;
  // Set CSS size for canvas
  canvas.style.width = `${width}px`;
  canvas.style.height = `${height}px`;

  // Scale drawing context to match device pixel ratio
  const ctx = canvas.getContext("2d")!;
  ctx.scale(dpr, dpr);
  // Enable image smoothing for better quality
  ctx.imageSmoothingEnabled = true;
  ctx.imageSmoothingQuality = "high";
}
