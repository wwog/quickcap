import { DPR, editToolGap, editToolHeight } from "../const";
import { isWindows } from "./device";

let promise: Promise<{
  imageData: ImageData;
  height: number;
  width: number;
}> | null = null;

export function exitApp() {
  window.app.exit();
}

export async function getScreenImageData() {
  if (promise) return promise;
  promise = new Promise<{
    imageData: ImageData;
    height: number;
    width: number;
  }>((resolve, reject) => {
    (async () => {
      try {
        const imageData = await window.app.getImage();
        const { width, height, arrayBuffer } = imageData;
        resolve({
          imageData: new ImageData(
            new Uint8ClampedArray(arrayBuffer),
            width,
            height
          ),
          height,
          width,
        });
      } catch (err) {
        reject(err);
      }
    })();
  });
  return promise;
}

// Calculate the position and size of the selection area based on the mouse movement
export const calcStartAndMove = ({
  startX,
  startY,
  moveX,
  moveY,
  maxX,
  maxY,
}: {
  startX: number;
  startY: number;
  moveX: number;
  moveY: number;
  maxX: number;
  maxY: number;
}) => {
  if (moveX >= 0 && moveY >= 0) {
    return {
      top: startY,
      left: startX,
      width: Math.min(maxX, moveX),
      height: Math.min(maxY, moveY),
    };
  }
  if (moveX < 0 && moveY >= 0) {
    return {
      top: startY,
      left: startX + moveX,
      width: Math.min(maxX, -moveX),
      height: Math.min(maxY, moveY),
    };
  }
  if (moveX >= 0 && moveY < 0) {
    return {
      top: startY + moveY,
      left: startX,
      width: Math.min(maxX, moveX),
      height: Math.min(maxY, -moveY),
    };
  }
  return {
    top: startY + moveY,
    left: startX + moveX,
    width: Math.min(maxX, -moveX),
    height: Math.min(maxY, -moveY),
  };
};

export const calcFixedPoint = ({
  resizeHandle,
  x,
  y,
  width,
  height,
}: {
  resizeHandle: string;
  x: number;
  y: number;
  width: number;
  height: number;
}) => {
  switch (resizeHandle) {
    case "resize-top-left":
      return {
        x: x + width,
        y: y + height,
      };
    case "resize-top":
    case "resize-top-right":
      return {
        x,
        y: y + height,
      };
    case "resize-right":
    case "resize-bottom-right":
    case "resize-bottom":
      return {
        x,
        y,
      };
    case "resize-bottom-left":
    case "resize-left":
      return {
        x: x + width,
        y,
      };
    default:
      return {
        x,
        y,
      };
  }
};

export const calcReactForResizing = ({
  resizeHandle,
  fixedX,
  fixedY,
  originWidth,
  originHeight,
  moveX,
  moveY,
  maxX,
  maxY,
}: {
  resizeHandle: string;
  fixedX: number;
  fixedY: number;
  originWidth: number;
  originHeight: number;
  moveX: number;
  moveY: number;
  maxX: number;
  maxY: number;
}) => {
  let actualMoveX = moveX;
  let actualMoveY = moveY;
  switch (resizeHandle) {
    case "resize-top-left":
      actualMoveX = moveX - originWidth;
      actualMoveY = moveY - originHeight;
      break;
    case "resize-top":
      actualMoveX = originWidth;
      actualMoveY = moveY - originHeight;
      break;
    case "resize-top-right":
      actualMoveX = moveX + originWidth;
      actualMoveY = moveY - originHeight;
      break;
    case "resize-right":
      actualMoveX = moveX + originWidth;
      actualMoveY = originHeight;
      break;
    case "resize-bottom-right":
      actualMoveX = moveX + originWidth;
      actualMoveY = moveY + originHeight;
      break;
    case "resize-bottom":
      actualMoveX = originWidth;
      actualMoveY = moveY + originHeight;
      break;
    case "resize-bottom-left":
      actualMoveX = moveX - originWidth;
      actualMoveY = moveY + originHeight;
      break;
    case "resize-left":
      actualMoveX = moveX - originWidth;
      actualMoveY = originHeight;
      break;
    default:
      break;
  }

  return calcStartAndMove({
    startX: fixedX,
    startY: fixedY,
    moveX: actualMoveX,
    moveY: actualMoveY,
    maxX,
    maxY,
  });
};

export const calcEditToolTop = (
  show = true,
  {
    x,
    y,
    height,
    width,
  }: {
    x: number;
    y: number;
    height: number;
    width: number;
  }
) => {
  const editTool = document.querySelector(".edit-tool") as HTMLElement;
  if (!editTool) {
    return;
  }
  if (!show) {
    editTool.style.visibility = "hidden";
    return;
  }
  if (editTool) {
    editTool.style.visibility = "visible";
    editTool.style.left = `${x + width - editTool.clientWidth}px`;
    const maxY = window.innerHeight;
    // const maxX = window.innerWidth;

    // under the selection area
    if (y + height + editToolHeight + editToolGap <= maxY) {
      editTool.style.top = `${y + height + editToolGap}px`;
      return;
    }
    // above the selection area
    if (y - editToolHeight - editToolGap >= 0) {
      editTool.style.top = `${y - editToolHeight - editToolGap}px`;
      return;
    }

    // inner the selection area
    editTool.style.top = `${y + height - editToolHeight - editToolGap}px`;
  }
};

export const matchWindow = ({
  x,
  y,
  windows,
}: {
  x: number;
  y: number;
  windows: {
    x: number;
    y: number;
    width: number;
    height: number;
  }[];
}) => {
  return windows.find((window) => {
    return (
      x >= window.x &&
      x <= window.x + window.width &&
      y >= window.y &&
      y <= window.y + window.height
    );
  });
};

export const getRectForWindow = ({
  x,
  y,
  width,
  height,
}: {
  x: number;
  y: number;
  width: number;
  height: number;
}) => {

  const rx = isWindows() ? x / DPR : x;
  const ry = isWindows() ? y / DPR : y;
  const rwidth = isWindows() ? width / DPR : width;
  const rheight = isWindows() ? height / DPR : height;

  const maxX = window.innerWidth;
  const maxY = window.innerHeight;
  const endX = Math.min(rx + rwidth, maxX);
  const endY = Math.min(ry + rheight, maxY);
  const startX = Math.max(rx, 0);
  const startY = Math.max(ry, 0);
  return {
    x: startX,
    y: startY,
    width: endX - startX,
    height: endY - startY,
  };
};

export function generateUID() {
  return `${Math.random().toString(36).substring(2, 10)}-${Math.random()
    .toString(36)
    .substring(2, 10)}-${Math.random().toString(36).substring(2, 10)}`;
}

/**
 * 返回圆内、且落在 [0, maxX]×[0, maxY] 范围内的所有整数像素点
 * @param {number} cx    圆心 x
 * @param {number} cy    圆心 y
 * @param {number} r     半径
 * @param {number} maxX  画布右边界（含）
 * @param {number} maxY  画布下边界（含）
 */
export function pointsInCircleClipped(cx: number, cy: number, r: number, maxX: number, maxY: number) {
  const pts = [];
  const r2 = r * r;

  // 外接正方形边界（先按圆算，再和画布求交，减少无效循环）
  const x0 = Math.max(0, Math.ceil(cx - r));
  const x1 = Math.min(maxX, Math.floor(cx + r));
  const y0 = Math.max(0, Math.ceil(cy - r));
  const y1 = Math.min(maxY, Math.floor(cy + r));

  for (let y = y0; y <= y1; y++) {
    const dy = y - cy;
    for (let x = x0; x <= x1; x++) {
      const dx = x - cx;
      if (dx * dx + dy * dy <= r2) {
        pts.push({ x, y });
      }
    }
  }
  return pts;
}

/**
 * 在两点之间生成均匀分布的插值点
 * @param {number} x1 起点x坐标
 * @param {number} y1 起点y坐标
 * @param {number} x2 终点x坐标
 * @param {number} y2 终点y坐标
 * @param {number} radius 圆半径，用于计算合适的插值点数量
 * @returns {Array<{x: number, y: number}>} 插值点数组
 */
export function interpolatePoints(x1: number, y1: number, x2: number, y2: number, radius: number) {
  const points: Array<{ x: number, y: number }> = [];

  // 计算两点之间的距离
  const dx = x2 - x1;
  const dy = y2 - y1;
  const distance = Math.sqrt(dx * dx + dy * dy);

  // 根据半径计算需要的插值点数量，确保圆之间有一定重叠
  // 重叠因子设为0.5，确保相邻圆之间有50%的重叠区域
  // 两个圆的中心距离应该是：2 * radius * (1 - overlapFactor)
  const overlapFactor = 0.3;
  const step = 2 * radius * (1 - overlapFactor);
  const numPoints = Math.max(2, Math.ceil(distance / step));

  // 生成插值点
  for (let i = 0; i < numPoints; i++) {
    const t = i / numPoints;
    const x = x1 + dx * t;
    const y = y1 + dy * t;
    points.push({ x, y });
  }
  points.push({ x: x2, y: y2 });

  return points;
}