import { DPR } from "../const";
import { isWindows, pointsInCircleClipped } from "../utils";
import type { TShapeMosaic } from "./editType";

const MOSAIC_SIZE = isWindows() ? 10 : 10 * DPR;

export class Mosaic {
  private imageData: ImageData;
  private drawData: ImageData;

  private canvas: HTMLCanvasElement;

  private cache: Map<string, { r: number; g: number; b: number; a: number }> =
    new Map();

  private width: number;
  private height: number;

  constructor({
    imgData,
    canvas,
  }: {
    imgData: ImageData;
    canvas: HTMLCanvasElement;
  }) {
    this.imageData = imgData;
    this.canvas = canvas;
    this.width = imgData.width;
    this.height = imgData.height;
    this.drawData = this.canvas
      .getContext("2d")!
      .getImageData(0, 0, this.width, this.height);
  }

  private getMosaicColor({
    y,
    x,
    cx,
    cy,
    r,
  }: {
    x: number;
    y: number;
    cx: number;
    cy: number;
    r: number;
  }) {
    // Calculate which mosaic block the current point is in
    const blockX = Math.floor(x / MOSAIC_SIZE);
    const blockY = Math.floor(y / MOSAIC_SIZE);

    const key = `${blockX},${blockY}`;
    if (this.cache.has(key)) {
      return this.cache.get(key);
    }

    // Calculate the actual boundaries of the mosaic block
    const blockStartX = blockX * MOSAIC_SIZE;
    const blockStartY = blockY * MOSAIC_SIZE;
    const blockEndX = Math.min(this.width, blockStartX + MOSAIC_SIZE);
    const blockEndY = Math.min(this.height, blockStartY + MOSAIC_SIZE);

    const radiusSquared = r * r;

    const data = this.imageData.data;

    let totalR = 0,
      totalG = 0,
      totalB = 0,
      totalA = 0;
    let pixelCount = 0;
    for (let y = blockStartY; y < blockEndY; y++) {
      // Pre-calculate dy^2
      const dy = y - cy;
      const dySquared = dy * dy;

      for (let x = blockStartX; x < blockEndX; x++) {
        // Calculate distance squared (avoid square root)
        const dx = x - cx;
        const distanceSquared = dx * dx + dySquared;

        // Check if inside the circular area
        if (distanceSquared <= radiusSquared) {
          const index = (y * this.width + x) * 4;
          totalR += data[index];
          totalG += data[index + 1];
          totalB += data[index + 2];
          totalA += data[index + 3];
          pixelCount++;
        }
      }
    }

    let avgColor;
    if (pixelCount > 0) {
      avgColor = {
        r: Math.floor(totalR / pixelCount),
        g: Math.floor(totalG / pixelCount),
        b: Math.floor(totalB / pixelCount),
        a: Math.floor(totalA / pixelCount),
      };
    } else {
      // If no pixels are in the circle, return transparent color
      avgColor = { r: 0, g: 0, b: 0, a: 0 };
    }

    // Store in cache
    this.cache.set(key, avgColor);
    return avgColor;
  }

  drawMosaicForCircle({
    cx,
    cy,
    r,
    fresh = false,
  }: {
    cx: number;
    cy: number;
    r: number;
    fresh?: boolean;
  }) {
    const pts = pointsInCircleClipped(
      cx * DPR,
      cy * DPR,
      r * DPR,
      this.imageData.width,
      this.imageData.height
    );

    pts.forEach(({ x, y }) => {
      const color = this.getMosaicColor({
        x,
        y,
        cx: cx * DPR,
        cy: cy * DPR,
        r: r * DPR,
      });
      if (color) {
        this.drawData.data[y * this.width * 4 + x * 4] = color.r;
        this.drawData.data[y * this.width * 4 + x * 4 + 1] = color.g;
        this.drawData.data[y * this.width * 4 + x * 4 + 2] = color.b;
        this.drawData.data[y * this.width * 4 + x * 4 + 3] = color.a;
      }
    });
    if (fresh) {
      this.canvas.getContext("2d")!.putImageData(this.drawData, 0, 0);
    }
  }

  drawMosaic(shape: TShapeMosaic) {
    const { path, radius } = shape.attr;
    path.forEach(({ x, y }) => {
      this.drawMosaicForCircle({ cx: x, cy: y, r: radius });
    });
    this.canvas.getContext("2d")!.putImageData(this.drawData, 0, 0);
  }

  clearMosaic() {
    this.canvas.getContext("2d")!.clearRect(0, 0, this.width, this.height);
    this.drawData = this.canvas
      .getContext("2d")!
      .getImageData(0, 0, this.width, this.height);
  }
}
