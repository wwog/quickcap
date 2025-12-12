import { DPR } from "../const";
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

    this.baseCanvas.style.position = "absolute";
    this.editCanvas.style.position = "absolute";
    this.baseCanvas.style.top = "0px";
    this.baseCanvas.style.left = "0px";
    this.editCanvas.style.left = "0px";
    this.editCanvas.style.left = "0px";
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

  setImg({
    img,
    x = 0,
    y = 0,
    width,
    height,
  }: {
    img: CanvasImageSource;
    x?: number;
    y?: number;
    width: number;
    height: number;
  }) {

    this.baseCtx.drawImage(
      img,
      x * DPR,
      y * DPR,
      width * DPR,
      height * DPR,
      0,
      0,
      width,
      height,
    );
  }
}
