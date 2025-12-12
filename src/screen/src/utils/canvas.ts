import { DPR } from "../const";

export function initCanvasSetting(
  canvas: HTMLCanvasElement,
  {
    width,
    height,
  }: {
    width: number;
    height: number;
    dpr?: number;
  }
) {
  // Set actual pixel size for canvas
  canvas.width = width * DPR;
  canvas.height = height * DPR;
  // Set CSS size for canvas
  canvas.style.width = `${width}px`;
  canvas.style.height = `${height}px`;

  // Scale drawing context to match device pixel ratio
  const ctx = canvas.getContext("2d")!;
  ctx.scale(DPR, DPR);
  // Enable image smoothing for better quality
  ctx.imageSmoothingEnabled = true;
  ctx.imageSmoothingQuality = "high";
}
