import { resizeHandles } from "../const";
import {
  bindDoubleClick,
  calcEditToolTop,
  calcFixedPoint,
  calcReactForResizing,
  calcStartAndMove,
  matchWindow,
} from "../utils";
import { SizeDisplay } from "./sizeDisplay";

export class DrawScreen {
  sizeDisplay: SizeDisplay;
  imgDom: HTMLImageElement;
  canvasContainer = document.createElement("div");
  baseCanvas = document.createElement("canvas");
  baseCtx: CanvasRenderingContext2D;
  maskCanvas = document.createElement("canvas");
  maskCtx: CanvasRenderingContext2D;
  editCanvas = document.createElement("canvas");
  editCtx: CanvasRenderingContext2D;

  selectRectDom: HTMLDivElement;

  isSelecting = false;
  isResizing = false;
  resizeHandle: string = "";

  // start point when mousedown
  startX = 0;
  startY = 0;

  // fixed point when resizing
  fixedX = 0;
  fixedY = 0;
  fixedWidth = 0;
  fixedHeight = 0;

  // Select area position and size
  selectX = 0;
  selectY = 0;
  selectWidth = 0;
  selectHeight = 0;

  mode: "select" | "waitEdit" | "edit" | "move" = "select";

  imgNaturalWidth = 0;
  imgNaturalHeight = 0;
  imgDrawWidth = 0;
  imgDrawHeight = 0;
  boxWidth = 0;
  boxHeight = 0;
  imgOffsetX = 0;
  imgOffsetY = 0;

  windows = [
    {
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
    },
    {
      x: 0,
      y: 0,
      width: 1062,
      height: 857,
    },
  ];

  constructor(appDom: HTMLDivElement, imgDom: HTMLImageElement) {
    this.imgDom = imgDom;
    this.canvasContainer.classList.add("canvas-container");
    this.baseCanvas.classList.add("base-canvas");
    this.maskCanvas.classList.add("mask-canvas");
    this.baseCtx = this.baseCanvas.getContext("2d") as CanvasRenderingContext2D;
    this.maskCtx = this.maskCanvas.getContext("2d") as CanvasRenderingContext2D;
    this.editCtx = this.editCanvas.getContext("2d") as CanvasRenderingContext2D;

    this.selectRectDom = document.createElement("div");
    this.selectRectDom.classList.add("select-rect");
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
    this.drawBase();
    this.drawMask();
    this.initListener();

    this.sizeDisplay = new SizeDisplay(appDom);
  }

  private initData = () => {
    this.imgNaturalWidth = this.imgDom.naturalWidth;
    this.imgNaturalHeight = this.imgDom.naturalHeight;
    this.boxWidth = this.canvasContainer.clientWidth;
    this.boxHeight = this.canvasContainer.clientHeight;

    // Consider device pixel ratio
    const dpr = window.devicePixelRatio || 1;

    // Set actual pixel size for canvas
    this.baseCanvas.width = this.boxWidth * dpr;
    this.baseCanvas.height = this.boxHeight * dpr;
    this.maskCanvas.width = this.boxWidth * dpr;
    this.maskCanvas.height = this.boxHeight * dpr;
    this.editCanvas.width = this.boxWidth * dpr;
    this.editCanvas.height = this.boxHeight * dpr;

    // Set CSS size for canvas
    this.baseCanvas.style.width = `${this.boxWidth}px`;
    this.baseCanvas.style.height = `${this.boxHeight}px`;
    this.maskCanvas.style.width = `${this.boxWidth}px`;
    this.maskCanvas.style.height = `${this.boxHeight}px`;
    this.editCanvas.style.width = `${this.boxWidth}px`;
    this.editCanvas.style.height = `${this.boxHeight}px`;

    // Scale drawing context to match device pixel ratio
    this.baseCtx.scale(dpr, dpr);
    this.maskCtx.scale(dpr, dpr);
    this.editCtx.scale(dpr, dpr);

    // Enable image smoothing for better quality
    this.baseCtx.imageSmoothingEnabled = true;
    this.baseCtx.imageSmoothingQuality = "high";
    this.maskCtx.imageSmoothingEnabled = true;
    this.maskCtx.imageSmoothingQuality = "high";

    const rateX = this.imgNaturalWidth / this.boxWidth;
    const rateY = this.imgNaturalHeight / this.boxHeight;

    const rate = Math.max(rateX, rateY);
    this.imgDrawWidth = this.imgNaturalWidth / rate;
    this.imgDrawHeight = this.imgNaturalHeight / rate;

    this.imgOffsetX = (this.boxWidth - this.imgDrawWidth) / 2;
    this.imgOffsetY = (this.boxHeight - this.imgDrawHeight) / 2;
  };

  private drawBase = () => {
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
      this.selectRectDom.style.left = `${this.selectX}px`;
      this.selectRectDom.style.top = `${this.selectY}px`;
      this.selectRectDom.style.width = `${this.selectWidth}px`;
      this.selectRectDom.style.height = `${this.selectHeight}px`;
    }
  };

  private selectStart = (e: MouseEvent) => {
    if (this.mode !== "select") {
      return;
    }
    this.isSelecting = true;
    this.startX = e.clientX;
    this.startY = e.clientY;
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
      if (!window) {
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
    this.selectRectDom.classList.add("select-rect-wait-edit");
  };

  private resizeStart = (e: MouseEvent) => {
    if (this.mode === "select") {
      return;
    }
    if ((e.target as HTMLDivElement)?.classList?.contains("resize-handle")) {
      this.isResizing = true;
      this.resizeHandle = (e.target as HTMLDivElement).dataset.role || "";
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
      this.canvasContainer.style.cursor =
        getComputedStyle(e.target as HTMLDivElement).cursor || "";
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
      this.canvasContainer.style.cursor = "move";
    }
  };
  private resizeMove = (e: MouseEvent) => {
    if (this.mode === "select") {
      return;
    }

    if (this.mode === "waitEdit" && this.resizeHandle) {
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
    this.canvasContainer.style.cursor = "";
    this.resizeHandle = "";
  };

  private onMouseDown = (e: MouseEvent) => {
    const isSelectRect =
      e.target === this.selectRectDom ||
      this.selectRectDom.contains(e.target as HTMLElement);
    calcEditToolTop(!isSelectRect, {
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
    switch (this.mode) {
      case "select":
        e.stopPropagation();
        e.preventDefault();
        this.selectEnd();
        calcEditToolTop(true, {
          x: this.selectX,
          y: this.selectY,
          width: this.selectWidth,
          height: this.selectHeight,
        });
        break;
      case "waitEdit":
      case "move":
        e.stopPropagation();
        e.preventDefault();
        this.resizeEnd();
        calcEditToolTop(true, {
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
    console.log(
      `%c🎄 mouseout`,
      "background-color: #00b548; color: #fff;padding: 2px 4px;border-radius: 2px;",
      e
    );
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
    this.canvasContainer.addEventListener("mousedown", this.onMouseDown);
    this.canvasContainer.addEventListener("mousemove", this.onMouseMove);
    this.canvasContainer.addEventListener("mouseup", this.onMouseUp);
    this.canvasContainer.addEventListener("mouseleave", this.onMouseLeave);

    bindDoubleClick(this.selectRectDom, () => {
      alert("double click");
    });
  };
}
