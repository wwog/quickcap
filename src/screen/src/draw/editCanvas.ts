import { initCanvasSetting } from "../utils/canvas";
export type EditCanvasMode = "normal" | "edit" | "drag";
export class EditCanvas {
  private baseCanvas: HTMLCanvasElement;
  private editCanvas: HTMLCanvasElement;

  private baseCtx: CanvasRenderingContext2D;
  private editCtx: CanvasRenderingContext2D;

  private _mode: EditCanvasMode = "normal";

  get mode() {
    return this._mode;
  }

  private set mode(mode: EditCanvasMode) {
    if (this._mode === mode) {
      return;
    }
    this._mode = mode;
  }

  constructor() {
    this.baseCanvas = document.createElement("canvas");
    this.editCanvas = document.createElement("canvas");
    this.baseCtx = this.baseCanvas.getContext("2d") as CanvasRenderingContext2D;
    this.editCtx = this.editCanvas.getContext("2d") as CanvasRenderingContext2D;
  }

  initCanvasSetting(width: number, height: number) {
    initCanvasSetting(this.baseCanvas, {
      width,
      height,
    });
    initCanvasSetting(this.editCanvas, {
      width,
      height,
    });
  }

  setParentDom(parentDom: HTMLElement) {
    parentDom.appendChild(this.baseCanvas);
    parentDom.appendChild(this.editCanvas);
  }

  setMode(mode: EditCanvasMode) {
    this.mode = mode;
  }

  getCtx() {
    return this.editCtx;
  }

  setImg(imgDom: HTMLImageElement) {
    this.baseCtx.drawImage(
      imgDom,
      0,
      0,
      imgDom.naturalWidth,
      imgDom.naturalHeight,
      0,
      0,
      Number(this.baseCanvas.style.width ? this.baseCanvas.style.width.replace("px", "") : imgDom.naturalWidth),
      Number(this.baseCanvas.style.height ? this.baseCanvas.style.height.replace("px", "") : imgDom.naturalHeight)
    );
  }
}