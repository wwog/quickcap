import { editToolGap, editToolHeight } from "../const";

let promise: Promise<{
  imageData: ImageData;
  height: number;
  width: number;
}> | null = null;

export function exitApp() {
  // (window as any).ipc.postMessage("escape_pressed");
  (window as any).app.exit();
}

export async function getScreenImageData() {
  if (promise) return promise;
  /* promise = new Promise((resolve, reject) => {
    try {
      (window as any).onRegionData = function (result: any) {
        console.log("Region data received:", result);
        if (result.error) {
          console.error("Error:", result.error);
          reject(result.error);
        } else {
          console.log(
            "Region extracted:",
            result.x,
            result.y,
            result.w,
            result.h
          );
          console.log("Data length:", result.data.length);
          // 这里可以处理返回的区域数据
          // result.data 是 base64 编码的 RGBA 数据
          resolve(result);
        }
      };
      (window as any).ipc.postMessage(JSON.stringify([x, y, width, height]));
    } catch (err) {
      reject(err);
    }
  }); */
  promise = new Promise<{
    imageData: ImageData;
    height: number;
    width: number;
  }>((resolve, reject) => {
    (async () => {
      try {
        const imageData = await (window as any).app.getImage();
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
  const maxX = window.innerWidth;
  const maxY = window.innerHeight;
  const endX = Math.min(x + width, maxX);
  const endY = Math.min(y + height, maxY);
  const startX = Math.max(x, 0);
  const startY = Math.max(y, 0);
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