import { DPR } from "../const";
import { generateUID, interpolatePoints } from "../utils";
import {
  calculateEllipseFromRect,
  calculateRectFromPoints,
  // drawArrow,
  // drawCircle,
  // drawPath,
  // drawRect,
  drawShape,
  initCanvasSetting,
} from "../utils/canvas";
import type { EditCanvasMode, TShape } from "./editType";
import { Mosaic } from "./mosaic";
import { ResizeAssist } from "./resizeAssist";

export class EditCanvas {
  private lastImg: null | {
    x: number;
    y: number;
    width: number;
    height: number;
  } = null;
  private baseCanvas: HTMLCanvasElement;
  private mosaicCanvas: HTMLCanvasElement;
  private editCanvas: HTMLCanvasElement;

  private baseCtx: CanvasRenderingContext2D;
  private mosaicCtx: CanvasRenderingContext2D;
  private editCtx: CanvasRenderingContext2D;

  private mosaic: Mosaic | null = null;

  private _mode: EditCanvasMode = "normal";

  private resizeAssist: ResizeAssist;

  private drawState: null | TShape = null;

  private shapeArr: TShape[] = [];

  private currentDrawPos = {
    x1: 0,
    y1: 0,
    x2: 0,
    y2: 0,
  };

  private drawing = false;

  // private dragging = false;

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
    this.mosaicCanvas = document.createElement("canvas");
    this.editCanvas = document.createElement("canvas");
    this.baseCtx = this.baseCanvas.getContext("2d") as CanvasRenderingContext2D;
    this.mosaicCtx = this.mosaicCanvas.getContext(
      "2d"
    ) as CanvasRenderingContext2D;
    this.editCtx = this.editCanvas.getContext("2d") as CanvasRenderingContext2D;

    this.baseCanvas.style.position = "absolute";
    this.mosaicCanvas.style.position = "absolute";
    this.editCanvas.style.position = "absolute";
    this.baseCanvas.style.top = "0px";
    this.baseCanvas.style.left = "0px";
    this.mosaicCanvas.style.top = "0px";
    this.mosaicCanvas.style.left = "0px";
    this.editCanvas.style.left = "0px";
    this.editCanvas.style.left = "0px";

    this.resizeAssist = new ResizeAssist();

    console.log("editCanvas created", this.resizeAssist);

    (window as any).editCanvas = this;
  }

  private getCanvasPos = (clientX: number, clientY: number) => {
    if (!this.lastImg) {
      return {
        x: 0,
        y: 0,
      };
    }
    return {
      x: Math.min(
        Math.max(clientX - this.lastImg.x, 0),
        this.lastImg.x + this.lastImg.width
      ),
      y: Math.min(
        Math.max(clientY - this.lastImg.y, 0),
        this.lastImg.y + this.lastImg.height
      ),
    };
  };

  private initListener = () => {
    this.editCanvas.addEventListener("mousedown", (e) => {
      if (this.mode === "normal") {
        return;
      }

      const { x, y } = this.getCanvasPos(e.clientX, e.clientY);
      this.currentDrawPos = {
        x1: x,
        y1: y,
        x2: x,
        y2: y,
      };
      this.drawing = true;
    });
    document.body.addEventListener("mousemove", (e) => {
      if (this.mode === "normal") {
        return;
      }

      if (this.drawing) {
        const { x, y } = this.getCanvasPos(e.clientX, e.clientY);
        this.currentDrawPos.x2 = x;
        this.currentDrawPos.y2 = y;

        switch (this.drawState?.shape) {
          case "rect":
            const rect = calculateRectFromPoints(this.currentDrawPos, {
              maxX: this.lastImg!.width,
              maxY: this.lastImg!.height,
            });
            this.drawState.attr = rect;
            this.renderAll();
            break;
          case "circle":
            const ellipse = calculateEllipseFromRect(this.currentDrawPos, {
              maxX: this.lastImg!.width,
              maxY: this.lastImg!.height,
            });
            this.drawState.attr = ellipse;
            this.renderAll();
            break;
          case "path":
            if (!this.drawState.attr.path.length) {
              this.drawState.attr.path.push({
                x: this.currentDrawPos.x1,
                y: this.currentDrawPos.y1,
              });
            }
            this.drawState.attr.path.push({ x: x, y: y });
            this.renderPreview();
            break;
          case "mosaic":
            console.log(
              `%c🎄 mosaic`,
              "background-color: #00b548; color: #fff;padding: 2px 4px;border-radius: 2px;",
              x,
              y,
              this.mosaic
            );
            if (!this.drawState.attr.path.length) {
              this.drawState.attr.path.push({
                x: this.currentDrawPos.x1,
                y: this.currentDrawPos.y1,
              });
              this.mosaic?.drawMosaicForCircle({
                cx: this.currentDrawPos.x1,
                cy: this.currentDrawPos.y1,
                r: this.drawState.attr.radius,
                fresh: true,
              });
            } else {
              // 获取上一个记录的点
              const lastPoint = this.drawState.attr.path[this.drawState.attr.path.length - 1];
              
              // 在当前点和上一个点之间生成插值点
              const interpolatedPoints = interpolatePoints(
                lastPoint.x,
                lastPoint.y,
                x,
                y,
                this.drawState.attr.radius
              );
              
              // 跳过第一个点（已经处理过）
              for (let i = 1; i < interpolatedPoints.length; i++) {
                const interpolatedPoint = interpolatedPoints[i];
                this.drawState.attr.path.push({
                  x: interpolatedPoint.x,
                  y: interpolatedPoint.y,
                });
                this.mosaic?.drawMosaicForCircle({
                  cx: interpolatedPoint.x,
                  cy: interpolatedPoint.y,
                  r: this.drawState.attr.radius,
                  fresh: true,
                });
              }
            }
            break;
          case "arrow":
            this.drawState.attr = {
              fromX: this.currentDrawPos.x1,
              fromY: this.currentDrawPos.y1,
              toX: this.currentDrawPos.x2,
              toY: this.currentDrawPos.y2,
            };
            this.renderAll();
            break;
          default:
            break;
        }
      }
    });
    document.body.addEventListener("mouseup", (e) => {
      console.log("🚀 ~ EditCanvas ~ e:", e);
      if (this.mode === "normal") {
        return;
      }
      if (this.drawing && this.drawState) {
        this.drawing = false;
        if (
          this.currentDrawPos.x1 !== this.currentDrawPos.x2 ||
          this.currentDrawPos.y1 !== this.currentDrawPos.y2
        ) {
          e.preventDefault();
          e.stopPropagation();
          this.shapeArr.push(this.drawState);
          this.setShape(this.drawState.shape);
        }
      }
    });
  };

  initCanvasSetting(width: number, height: number) {
    initCanvasSetting(this.baseCanvas, {
      width,
      height,
    });
    initCanvasSetting(this.mosaicCanvas, {
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
    parentDom.appendChild(this.mosaicCanvas);
    parentDom.appendChild(this.editCanvas);
  }

  setMode(mode: EditCanvasMode) {
    this.mode = mode;
  }

  getCtx() {
    return this.editCtx;
  }

  private async getImageDataUrl() {
    this.baseCtx.drawImage(
      this.mosaicCanvas,
      0,
      0,
      this.mosaicCanvas.width,
      this.mosaicCanvas.height,
      0,
      0,
      this.lastImg!.width,
      this.lastImg!.height
    );
    this.baseCtx.drawImage(
      this.editCanvas,
      0,
      0,
      this.editCanvas.width,
      this.editCanvas.height,
      0,
      0,
      this.lastImg!.width,
      this.lastImg!.height
    );
    // return this.baseCanvas.toDataURL("image/png");
    return new Promise((resolve) => {
      this.baseCanvas.toBlob((blob) => {
        if (blob) {
          resolve(blob.arrayBuffer());
        }
      });
    })
  }

  writeToClipboard = async () => {
    console.log("writeToClipboard");
    const dataURL = await this.getImageDataUrl();
    (window as any).app.copyToClipboard(dataURL);
    /*  await this.baseCanvas.toBlob(async (blob) => {
      if (blob) {
        try {
          // 将blob转换为ArrayBuffer，然后通过IPC发送
          const arrayBuffer = await blob.arrayBuffer();
          const uint8Array = new Uint8Array(arrayBuffer);
          
          // 使用IPC剪切板API
          if ((window as any).app && (window as any).app.copyToClipboard) {
            await (window as any).app.copyToClipboard(uint8Array);
            console.log("图片已通过IPC复制到剪贴板");
            (window as any).app.exit();
          } else {
            // 降级方案：在新窗口中打开图片，用户可以手动保存
            const url = URL.createObjectURL(blob);
            const newWindow = window.open(url, '_blank');
            if (newWindow) {
              alert("由于浏览器安全限制，无法直接复制到剪贴板。图片已在新窗口中打开，您可以右键保存图片或手动复制。");
            } else {
              // 如果连新窗口都无法打开，则提供下载链接
              const a = document.createElement('a');
              a.href = url;
              a.download = `screenshot_${Date.now()}.png`;
              document.body.appendChild(a);
              a.click();
              document.body.removeChild(a);
              alert("由于浏览器安全限制，无法直接复制到剪贴板。图片已开始下载。");
            }
          }
        } catch (error) {
          console.error("复制到剪贴板失败:", error);
          // 降级方案：创建下载链接
          const url = URL.createObjectURL(blob);
          const a = document.createElement('a');
          a.href = url;
          a.download = `screenshot_${Date.now()}.png`;
          document.body.appendChild(a);
          a.click();
          document.body.removeChild(a);
          alert("复制到剪贴板失败，图片已开始下载。");
        }
      }
    }, "image/png"); */
  };

  saveImageToFolder = async () => {
    console.log("saveImageToFolder");
    const dataURL = await this.getImageDataUrl();
    (window as any).app.saveImageToFolder(dataURL);
  };

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
    if (
      this.lastImg &&
      this.lastImg.x === x &&
      this.lastImg.y === y &&
      this.lastImg.width === width &&
      this.lastImg.height === height
    ) {
      return;
    }
    const t0 = performance.now();
    const imgData = (img as HTMLCanvasElement)
      .getContext("2d")!
      .getImageData(x * DPR, y * DPR, width * DPR, height * DPR);
    this.baseCtx.putImageData(imgData, 0, 0);

    /* this.baseCtx.drawImage(
      img,
      x * DPR,
      y * DPR,
      width * DPR,
      height * DPR,
      0,
      0,
      width,
      height
    ); */

    const t1 = performance.now();
    console.log(`drawImage cost ${t1 - t0} ms`);
    this.lastImg = {
      x,
      y,
      width,
      height,
    };

    this.initListener();

    this.mosaic = new Mosaic({
      imgData,
      canvas: this.mosaicCanvas,
    });
  }

  setShape(shape = "rect") {
    switch (shape) {
      case "rect":
        this.drawState = {
          id: generateUID(),
          shape: "rect",
          attr: {
            x: 0,
            y: 0,
            width: 0,
            height: 0,
          },
          pen: {
            color: "#ff0000",
            lineWidth: 2,
          },
        };
        break;
      case "circle":
        this.drawState = {
          id: generateUID(),
          shape: "circle",
          attr: {
            centerX: 0,
            centerY: 0,
            radiusX: 0,
            radiusY: 0,
            isCircle: false,
            left: 0,
            top: 0,
            width: 0,
            height: 0,
          },
          pen: {
            color: "#ff0000",
            lineWidth: 2,
          },
        };
        break;
      case "path":
        this.drawState = {
          id: generateUID(),
          shape: "path",
          attr: {
            path: [],
          },
          pen: {
            color: "#ff0000",
            lineWidth: 2,
          },
        };
        break;
      case "mosaic":
        this.drawState = {
          id: generateUID(),
          shape: "mosaic",
          attr: {
            path: [],
            radius: 10,
          },
        };
        break;
      case "arrow":
        this.drawState = {
          id: generateUID(),
          shape: "arrow",
          attr: {
            fromX: 0,
            fromY: 0,
            toX: 0,
            toY: 0,
          },
          pen: {
            color: "#ff0000",
            lineWidth: 2,
          },
        };
        break;
      default:
        this.drawState = null;
        break;
    }
  }

  private renderAll = () => {
    this.editCtx.clearRect(0, 0, this.editCanvas.width, this.editCanvas.height);
    this.shapeArr.forEach((shape) => {
      drawShape(this.editCtx, shape);
    });
    this.renderPreview();
  };
  private renderPreview = () => {
    if (
      this.currentDrawPos.x1 === this.currentDrawPos.x2 &&
      this.currentDrawPos.y1 === this.currentDrawPos.y2
    ) {
      return;
    }
    if (!this.drawState) {
      return;
    }
    drawShape(this.editCtx, this.drawState);
  };
}
