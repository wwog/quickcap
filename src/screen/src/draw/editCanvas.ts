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

  writeToClipboard = async () => {
    console.log('writeToClipboard');
    await this.baseCanvas.toBlob(async (blob) => {
      console.log("🚀 ~ EditCanvas ~ blob:", blob);
      if (blob) {
        try {
          // 将blob转换为ArrayBuffer，然后通过IPC发送
          const arrayBuffer = await blob.arrayBuffer();
          const uint8Array = new Uint8Array(arrayBuffer);
          
          // 使用IPC剪切板API
          if ((window as any).app && (window as any).app.copyToClipboard) {
            await (window as any).app.copyToClipboard(uint8Array);
            console.log("图片已通过IPC复制到剪贴板");
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
    });
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
