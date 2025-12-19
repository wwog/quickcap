import { resizeHandles } from "../const";
import {
  bindDoubleClick,
  // calcEditToolTop,
  calcFixedPoint,
  calcReactForResizing,
  calcStartAndMove,
  exitApp,
  getRectForWindow,
  matchWindow,
} from "../utils";
import { initCanvasSetting } from "../utils/canvas";
import { EditCanvas } from "./editCanvas";
import { EditTools } from "./editTools";
// import { onClickFinish } from "./func";
import { SizeDisplay } from "./sizeDisplay";

type TMode = "select" | "waitEdit" | "resizing" | "edit" | "move";

/**
 * Manages interface drawing and interaction, including selection, editing, moving, etc.
 * Do not put exported methods here, put them in func.ts.
 */
export class DrawScreen {
  private sizeDisplay: SizeDisplay;
  private editTools: EditTools;

  private editCanvas: EditCanvas;

  private imgDom: HTMLImageElement | null;
  private canvasContainer = document.createElement("div");
  private baseCanvas = document.createElement("canvas");
  private maskCanvas = document.createElement("canvas");
  private baseCtx: CanvasRenderingContext2D;
  private maskCtx: CanvasRenderingContext2D;

  private selectRectDom: HTMLDivElement;

  private isSelecting = false;
  private resizeHandle: string = "";

  // start point when mousedown
  private startX = 0;
  private startY = 0;

  // fixed point when resizing
  private fixedX = 0;
  private fixedY = 0;
  private fixedWidth = 0;
  private fixedHeight = 0;

  // Select area position and size
  private selectX = 0;
  private selectY = 0;
  private selectWidth = 0;
  private selectHeight = 0;

  private matchedWindow: {
    x: number;
    y: number;
    width: number;
    height: number;
  } | undefined = undefined;

  private _mode: TMode = "select";

  /**
   * 获取当前模式
   */
  public get mode(): TMode {
    return this._mode;
  }

  /**
   * 设置模式
   */
  public set mode(value: TMode) {
    const oldMode = this._mode;
    if (oldMode !== value) {
      this._mode = value;
      switch (value) {
        case "waitEdit": {
          this.canvasContainer.style.cursor = "";
          this.selectRectDom.style.cursor = "move";
          break;
        }
        case "move": {
          this.canvasContainer.style.cursor = "move";
          break;
        }
        case "edit": {
          this.canvasContainer.style.cursor = "";
          this.selectRectDom.style.cursor = "crosshair";
          this.canvasContainer.classList.add("edit-mode");
          break;
        }
      }
    }
  }

  private imgNaturalWidth = 0;
  private imgNaturalHeight = 0;
  private imgDrawWidth = 0;
  private imgDrawHeight = 0;
  private boxWidth = 0;
  private boxHeight = 0;
  private imgOffsetX = 0;
  private imgOffsetY = 0;

  private windows: {
    x: number;
    y: number;
    width: number;
    height: number;
  }[] = [
    /* {
      x: 20,
      y: 148,
      width: 799,
      height: 521,
    },
    {
      x: 198,
      y: 148,
      width: 799,
      height: 521,
    }, */
  ];

  constructor(appDom: HTMLDivElement) {
    // this.imgDom = imgDom;
    this.imgDom = null;
    this.canvasContainer.classList.add("canvas-container");
    this.maskCanvas.classList.add("mask-canvas");
    this.maskCtx = this.maskCanvas.getContext("2d") as CanvasRenderingContext2D;
    this.baseCtx = this.baseCanvas.getContext("2d") as CanvasRenderingContext2D;
    this.editCanvas = new EditCanvas();

    this.selectRectDom = document.createElement("div");
    this.selectRectDom.classList.add("select-rect");
    // this.selectRectDom.appendChild(this.editCanvas);
    appDom.appendChild(this.canvasContainer);
    this.canvasContainer.appendChild(this.baseCanvas);
    this.canvasContainer.appendChild(this.maskCanvas);

    // Initialize resize handles
    resizeHandles.forEach((handleName) => {
      const handleDom = document.createElement("div");
      handleDom.classList.add("resize-handle", handleName);
      handleDom.dataset.role = handleName;
      this.selectRectDom.appendChild(handleDom);
    });

    this.initData();

    this.drawMask();
    this.initListener();

    this.sizeDisplay = new SizeDisplay(this.canvasContainer);
    this.editTools = new EditTools();

    this.editTools.addListener([
      {
        role: "edit",
        listener: (shape?: string) => {
          this.setEditCanvasBg();
          this.editCanvas.setMode("edit");
          const newShape = shape === this.editTools.active ? "" : shape || "";
          this.editCanvas.setShape(newShape);
          this.editTools.active = newShape;
        },
      },
      {
        role: "download",
        listener: () => {
          this.setEditCanvasBg();
          this.editCanvas.saveImageToFolder();
        },
      },
      {
        role: "finish",
        listener: () => {
          this.setEditCanvasBg();
          this.editCanvas.writeToClipboard();
        },
      },
      {
        role: "cancel",
        listener: () => {
          exitApp();
        },
      },
    ]);

    (window as any).app
      ?.getWindows?.()
      .then(
        (
          windows: {
            name: string;
            bounds: {
              x: number;
              y: number;
              width: number;
              height: number;
            };
          }[]
        ) => {
          const maxX = window.innerWidth;
          const maxY = window.innerHeight;
          const arr: {
            x: number;
            y: number;
            width: number;
            height: number;
          }[] = [];
          windows.forEach((win) => {
            if (win.name !== "tao window") {
              const { x, y, width, height } = win.bounds;

              arr.push(getRectForWindow({ x, y, width, height }));
            }
          });
          // Add the main window area
          arr.push({
            x: 0,
            y: 0,
            width: maxX,
            height: maxY,
          });
          this.windows = arr;
        }
      )
      .catch((err: any) => console.error(err));
  }

  setImgDom = (imgDom: HTMLImageElement) => {
    this.imgDom = imgDom;
    this.imgNaturalWidth = this.imgDom.naturalWidth;
    this.imgNaturalHeight = this.imgDom.naturalHeight;
    const rateX = this.imgNaturalWidth / this.boxWidth;
    const rateY = this.imgNaturalHeight / this.boxHeight;

    const rate = Math.max(rateX, rateY);
    this.imgDrawWidth = this.imgNaturalWidth / rate;
    this.imgDrawHeight = this.imgNaturalHeight / rate;

    this.imgOffsetX = (this.boxWidth - this.imgDrawWidth) / 2;
    this.imgOffsetY = (this.boxHeight - this.imgDrawHeight) / 2;

    this.drawBase();
  };

  private setEditCanvasBg = () => {
    if (this.mode === "edit") {
      return;
    }
    this.mode = "edit";
    this.editCanvas.initCanvasSetting(this.selectWidth, this.selectHeight);
    this.editCanvas.setParentDom(this.selectRectDom);
    this.editCanvas.setImg({
      img: this.baseCanvas,
      x: this.selectX,
      y: this.selectY,
      width: this.selectWidth,
      height: this.selectHeight,
    });
  };

  private drawBase = () => {
    if (!this.imgDom) {
      console.error("Image dom is not set");
      return;
    }
    // Draw image
    this.baseCtx.drawImage(
      this.imgDom,
      0,
      0,
      this.imgNaturalWidth,
      this.imgNaturalHeight,
      this.imgOffsetX,
      this.imgOffsetY,
      this.imgDrawWidth,
      this.imgDrawHeight
    );
  };

  private initData = () => {
    this.boxWidth = this.canvasContainer.clientWidth;
    this.boxHeight = this.canvasContainer.clientHeight;

    initCanvasSetting(this.maskCanvas, {
      width: this.boxWidth,
      height: this.boxHeight,
    });
    initCanvasSetting(this.baseCanvas, {
      width: this.boxWidth,
      height: this.boxHeight,
    });
  };

  private drawMask = () => {
    this.maskCtx.clearRect(0, 0, this.boxWidth, this.boxHeight);
    // Draw semi-transparent mask layer
    this.maskCtx.fillStyle = "rgba(0, 0, 0, 0.5)";
    this.maskCtx.fillRect(0, 0, this.boxWidth, this.boxHeight);

    // Show selection area
    if (this.selectWidth && this.selectHeight) {
      this.maskCtx.clearRect(
        this.selectX,
        this.selectY,
        this.selectWidth,
        this.selectHeight
      );
      this.canvasContainer.appendChild(this.selectRectDom);
      this.selectRectDom.style.left = `${this.selectX - 1}px`;
      this.selectRectDom.style.top = `${this.selectY - 1}px`;
      this.selectRectDom.style.width = `${this.selectWidth}px`;
      this.selectRectDom.style.height = `${this.selectHeight}px`;
    } else {
      this.selectRectDom.remove();
    }
  };

  private selectStart = (e: MouseEvent) => {
    if (this.mode !== "select") {
      return;
    }
    this.isSelecting = true;
    this.startX = e.clientX;
    this.startY = e.clientY;
    console.log(
      `%c🎄 select start`,
      "background-color: #00b548; color: #fff;padding: 2px 4px;border-radius: 2px;",
      this.startX,
      this.startY
    );
  };

  private selectMove = (e: MouseEvent) => {
    if (this.mode !== "select") {
      return;
    }

    if (!this.isSelecting) {
      const window = matchWindow({
        x: e.clientX,
        y: e.clientY,
        windows: this.windows,
      });
      this.matchedWindow = window;
      if (!window) {
        this.matchedWindow = undefined;
        return;
      }

      this.selectX = window.x;
      this.selectY = window.y;
      this.selectWidth = window.width;
      this.selectHeight = window.height;

      this.drawMask();

      return;
    }
    const { top, left, width, height } = calcStartAndMove({
      startX: this.startX,
      startY: this.startY,
      moveX: e.clientX - this.startX,
      moveY: e.clientY - this.startY,
      maxX: this.boxWidth,
      maxY: this.boxHeight,
    });

    this.selectX = left;
    this.selectY = top;
    this.selectWidth = width;
    this.selectHeight = height;

    this.drawMask();
  };

  private selectEnd = () => {
    if (this.mode !== "select") {
      return;
    }
    this.isSelecting = false;
    this.mode = "waitEdit";
  };

  private resizeStart = (e: MouseEvent) => {
    if (this.mode === "select") {
      return;
    }
    if ((e.target as HTMLDivElement)?.classList?.contains("resize-handle")) {
      this.resizeHandle = (e.target as HTMLDivElement).dataset.role || "";
      this.mode = "resizing";
      const cursor = getComputedStyle(e.target as HTMLDivElement).cursor || "";
      this.canvasContainer.style.cursor = cursor;
      this.selectRectDom.style.cursor = cursor;
      this.startX = e.clientX;
      this.startY = e.clientY;
      const { x, y } = calcFixedPoint({
        resizeHandle: this.resizeHandle,
        x: this.selectX,
        y: this.selectY,
        width: this.selectWidth,
        height: this.selectHeight,
      });
      this.fixedX = x;
      this.fixedY = y;
      this.fixedWidth = this.selectWidth;
      this.fixedHeight = this.selectHeight;
    } else if (
      (e.target as HTMLDivElement)?.classList?.contains("select-rect")
    ) {
      this.mode = "move";
      this.startX = e.clientX;
      this.startY = e.clientY;
      this.fixedX = this.selectX;
      this.fixedY = this.selectY;
      this.fixedWidth = this.selectWidth;
      this.fixedHeight = this.selectHeight;
    }
  };
  private resizeMove = (e: MouseEvent) => {
    if (this.mode === "select") {
      return;
    }

    if (this.mode === "resizing" && this.resizeHandle) {
      const { top, left, width, height } = calcReactForResizing({
        resizeHandle: this.resizeHandle,
        fixedX: this.fixedX,
        fixedY: this.fixedY,
        originWidth: this.fixedWidth,
        originHeight: this.fixedHeight,
        moveX: e.clientX - this.startX,
        moveY: e.clientY - this.startY,
        maxX: this.boxWidth,
        maxY: this.boxHeight,
      });
      this.selectX = left;
      this.selectY = top;
      this.selectWidth = width;
      this.selectHeight = height;
      this.drawMask();
    } else if (this.mode === "move") {
      const moveX = e.clientX - this.startX;
      const moveY = e.clientY - this.startY;
      let left = this.fixedX + moveX;
      let top = this.fixedY + moveY;
      if (left < 0 || left + this.fixedWidth > this.boxWidth) {
        this.fixedX = this.selectX;
        this.fixedWidth = this.selectWidth;
        this.startX = e.clientX;
      }
      if (top < 0 || top + this.fixedHeight > this.boxHeight) {
        this.fixedY = this.selectY;
        this.fixedHeight = this.selectHeight;
        this.startY = e.clientY;
      }
      if (left < 0) {
        left = 0;
      } else if (left + this.fixedWidth > this.boxWidth) {
        left = this.boxWidth - this.fixedWidth;
      }
      if (top < 0) {
        top = 0;
      } else if (top + this.fixedHeight > this.boxHeight) {
        top = this.boxHeight - this.fixedHeight;
      }
      this.selectX = left;
      this.selectY = top;
      this.drawMask();
    }
  };
  private resizeEnd = () => {
    if (this.mode === "select") {
      return;
    }
    this.mode = "waitEdit";
    this.resizeHandle = "";
  };

  private onMouseDown = (e: MouseEvent) => {
    console.log(
      `%c🎄 mouse down`,
      "background-color: #00b548; color: #fff;padding: 2px 4px;border-radius: 2px;",
      this.mode,
      e.clientX,
      e.clientY
    );
    if (this.mode === "edit") return;
    const isSelectRect =
      e.target === this.selectRectDom ||
      this.selectRectDom.contains(e.target as HTMLElement);
    this.editTools.render(this.mode === "waitEdit" && !isSelectRect, {
      x: this.selectX,
      y: this.selectY,
      width: this.selectWidth,
      height: this.selectHeight,
    });

    if (this.mode === "select") {
      this.selectStart(e);
    } else {
      this.resizeStart(e);
    }
  };

  private onMouseMove = (e: MouseEvent) => {
    console.log(
      `%c🎄 mouse move`,
      "background-color: #00b548; color: #fff;padding: 2px 4px;border-radius: 2px;",
      this.mode,
      e.clientX,
      e.clientY
    );
    if (this.mode === "edit") return;
    if (this.mode === "select") {
      this.selectMove(e);
      this.sizeDisplay.render(true, {
        x: this.selectX,
        y: this.selectY,
        width: this.selectWidth,
        height: this.selectHeight,
      });
    } else {
      this.resizeMove(e);
      this.sizeDisplay.render(true, {
        x: this.selectX,
        y: this.selectY,
        width: this.selectWidth,
        height: this.selectHeight,
      });
    }
  };

  private onMouseUp = (e: MouseEvent) => {
    console.log(
      `%c🎄 mouse up`,
      "background-color: #00b548; color: #fff;padding: 2px 4px;border-radius: 2px;",
      this.mode,
      this.startX,
      this.startY,
      e.clientX,
      e.clientY,
      this.selectWidth,
      this.selectHeight
    );
    if (this.mode === "edit") return;
    switch (this.mode) {
      case "select":
        e.stopPropagation();
        e.preventDefault();
        if (this.selectWidth && this.selectHeight) {
          this.selectEnd();
          this.editTools.render(true, {
            x: this.selectX,
            y: this.selectY,
            width: this.selectWidth,
            height: this.selectHeight,
          });
          this.drawMask();
        } else {
          if (this.matchedWindow) {
            this.selectX = this.matchedWindow.x;
            this.selectY = this.matchedWindow.y;
            this.selectWidth = this.matchedWindow.width;
            this.selectHeight = this.matchedWindow.height;
          }
          this.drawMask();
          this.isSelecting = false;
        }
        break;
      case "waitEdit":
      case "move":
      case "resizing":
        e.stopPropagation();
        e.preventDefault();
        this.resizeEnd();
        this.editTools.render(true, {
          x: this.selectX,
          y: this.selectY,
          width: this.selectWidth,
          height: this.selectHeight,
        });
        break;
      default:
        break;
    }
  };

  private onMouseLeave = (e: MouseEvent) => {
    /* console.log(
      `%c🎄 mouseout`,
      "background-color: #00b548; color: #fff;padding: 2px 4px;border-radius: 2px;",
      e
    ); */
    if (this.mode === "move") {
      const x = e.clientX;
      const y = e.clientY;

      if (x > this.boxWidth) {
        this.selectX = this.boxWidth - this.selectWidth;
      } else if (x < 0) {
        this.selectX = 0;
      }
      if (y > this.boxHeight) {
        this.selectY = this.boxHeight - this.selectHeight;
      } else if (y < 0) {
        this.selectY = 0;
      }

      this.fixedX = this.selectX;
      this.fixedY = this.selectY;
      this.fixedWidth = this.selectWidth;
      this.fixedHeight = this.selectHeight;
      this.startX = e.clientX;
      this.startY = e.clientY;
      this.drawMask();
    }
  };

  private initListener = () => {
    // Selection logic
    document.body.addEventListener("mousedown", this.onMouseDown);
    document.body.addEventListener("mousemove", this.onMouseMove);
    document.body.addEventListener("mouseup", this.onMouseUp);
    document.body.addEventListener("mouseleave", this.onMouseLeave);

    bindDoubleClick(this.selectRectDom, () => {
      console.log("================double click================");
      this.setEditCanvasBg();
      this.editCanvas.writeToClipboard();
    });
  };

  putImageData = ({
    imageData,
    width,
    height,
  }: {
    width: number;
    height: number;
    imageData: ImageData;
  }) => {
    this.imgNaturalWidth = width;
    this.imgNaturalHeight = height;
    const rateX = this.imgNaturalWidth / this.boxWidth;
    const rateY = this.imgNaturalHeight / this.boxHeight;

    const rate = Math.max(rateX, rateY);
    this.imgDrawWidth = this.imgNaturalWidth / rate;
    this.imgDrawHeight = this.imgNaturalHeight / rate;

    this.imgOffsetX = (this.boxWidth - this.imgDrawWidth) / 2;
    this.imgOffsetY = (this.boxHeight - this.imgDrawHeight) / 2;

    this.baseCtx.putImageData(imageData, 0, 0);
  };

  // getSelectedImg = () => {
  //   this.baseCanvas
  // };
}
