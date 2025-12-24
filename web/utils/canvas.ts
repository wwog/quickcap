import { DPR } from "../const";
import type {
  TShape,
  TShapeArrow,
  TShapeCircle,
  TShapePath,
  TShapeRect,
} from "../draw/editType";

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

// 由两个点计算椭圆参数
export function calculateEllipseFromRect(
  {
    x1,
    y1,
    x2,
    y2,
  }: {
    x1: number;
    y1: number;
    x2: number;
    y2: number;
  },
  {
    maxX,
    maxY,
  }: {
    maxX: number;
    maxY: number;
  }
) {
  // 确保 x1 < x2, y1 < y2
  const left = Math.min(x1, x2);
  const right = Math.min(Math.max(x1, x2), maxX);
  const top = Math.min(y1, y2);
  const bottom = Math.min(Math.max(y1, y2), maxY);

  const width = right - left;
  const height = bottom - top;

  // 中心点
  const centerX = left + width / 2;
  const centerY = top + height / 2;

  // 椭圆半径（半宽和半高）
  const radiusX = width / 2;
  const radiusY = height / 2;

  // 判断是否为圆形（允许一定的误差）
  const isCircle = Math.abs(width - height) < 2;

  return {
    centerX,
    centerY,
    radiusX,
    radiusY,
    isCircle,
    left,
    top,
    width,
    height,
  };
}

// 由两个点计算矩形参数
export function calculateRectFromPoints(
  {
    x1,
    y1,
    x2,
    y2,
  }: {
    x1: number;
    y1: number;
    x2: number;
    y2: number;
  },
  {
    maxX,
    maxY,
  }: {
    maxX: number;
    maxY: number;
  }
) {
  const left = Math.min(x1, x2);
  const right = Math.min(Math.max(x1, x2), maxX);
  const top = Math.min(y1, y2);
  const bottom = Math.min(Math.max(y1, y2), maxY);

  return {
    x: left,
    y: top,
    width: right - left,
    height: bottom - top,
  };
}

export function drawRect(ctx: CanvasRenderingContext2D, shape: TShapeRect) {
  ctx.strokeStyle = shape.pen.color;
  ctx.lineWidth = shape.pen.lineWidth;
  ctx.strokeRect(
    shape.attr.x,
    shape.attr.y,
    shape.attr.width,
    shape.attr.height
  );
}

export function drawCircle(ctx: CanvasRenderingContext2D, shape: TShapeCircle) {
  ctx.strokeStyle = shape.pen.color;
  ctx.lineWidth = shape.pen.lineWidth;
  const centerX = shape.attr.centerX;
  const centerY = shape.attr.centerY;
  const radiusX = shape.attr.radiusX;
  const radiusY = shape.attr.radiusY;

  ctx.beginPath();
  ctx.ellipse(centerX, centerY, radiusX, radiusY, 0, 0, Math.PI * 2);
  ctx.stroke();
}

export function drawPath(ctx: CanvasRenderingContext2D, shape: TShapePath) {
  console.log(
    "🚀 ~ drawPath ~ shape.attr.path.length:",
    shape.attr.path.length
  );
  if (shape.attr.path.length < 2) return;
  ctx.strokeStyle = shape.pen.color;
  // ctx.fillStyle = shape.pen.color;
  ctx.lineWidth = shape.pen.lineWidth;
  ctx.beginPath();
  ctx.moveTo(shape.attr.path[0].x, shape.attr.path[0].y);
  for (let i = 1; i < shape.attr.path.length; i++) {
    ctx.lineTo(shape.attr.path[i].x, shape.attr.path[i].y);
  }
  ctx.stroke();
}

export function drawArrow(ctx: CanvasRenderingContext2D, shape: TShapeArrow) {
  ctx.strokeStyle = shape.pen.color;
  // ctx.fillStyle = shape.pen.color;
  ctx.lineWidth = shape.pen.lineWidth;

  const { fromX, fromY, toX, toY } = shape.attr;

  // ctx.save();
  ctx.beginPath();

  // // 设置线条样式
  // ctx.lineWidth = lineWidth;
  // ctx.strokeStyle = color;
  // ctx.lineCap = lineCap;

  const headLength = 20; // 箭头长度
  const headAngle = Math.PI / 12; // 箭头角度（30度）
  const fillArrow = true   // 是否填充箭头

  // 计算线段角度
  const angle = Math.atan2(toY - fromY, toX - fromX);
  const length = Math.sqrt(Math.pow(toX - fromX, 2) + Math.pow(toY - fromY, 2));

  // 绘制主线
  ctx.moveTo(fromX, fromY);

  // 如果箭头比线段还长，调整箭头长度
  const effectiveHeadLength = Math.min(headLength, length * 0.5);
  const mainLineEndX = toX - Math.cos(angle) * (effectiveHeadLength - 2);
  const mainLineEndY = toY - Math.sin(angle) * (effectiveHeadLength - 2);

  ctx.lineTo(mainLineEndX, mainLineEndY);
  ctx.stroke();

  // 绘制箭头
  ctx.beginPath();
  ctx.moveTo(toX, toY);

  // 计算箭头两侧点
  const arrowX1 = toX - effectiveHeadLength * Math.cos(angle - headAngle);
  const arrowY1 = toY - effectiveHeadLength * Math.sin(angle - headAngle);

  const arrowX2 = toX - effectiveHeadLength * Math.cos(angle + headAngle);
  const arrowY2 = toY - effectiveHeadLength * Math.sin(angle + headAngle);

  // 绘制箭头三角形
  ctx.lineTo(arrowX1, arrowY1);
  ctx.lineTo(arrowX2, arrowY2);
  ctx.closePath();

  if (fillArrow) {
    ctx.fillStyle = shape.pen.color;
    ctx.fill();
  } else {
    ctx.stroke();
  }
}

export function drawShape(ctx: CanvasRenderingContext2D, shape: TShape) {
  switch (shape.shape) {
    case "rect":
      drawRect(ctx, shape);
      break;
    case "circle":
      drawCircle(ctx, shape);
      break;
    case "path":
      drawPath(ctx, shape);
      break;
    case "arrow":
      drawArrow(ctx, shape);
      break;
  }
}
