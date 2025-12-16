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
    case "top-left":
      return {
        x: x + width,
        y: y + height,
      };
    case "top":
    case "top-right":
      return {
        x,
        y: y + height,
      };
    case "right":
    case "bottom-right":
    case "bottom":
      return {
        x,
        y,
      };
    case "bottom-left":
    case "left":
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
    case "top-left":
      actualMoveX = moveX - originWidth;
      actualMoveY = moveY - originHeight;
      break;
    case "top":
      actualMoveX = originWidth;
      actualMoveY = moveY - originHeight;
      break;
    case "top-right":
      actualMoveX = moveX + originWidth;
      actualMoveY = moveY - originHeight;
      break;
    case "right":
      actualMoveX = moveX + originWidth;
      actualMoveY = originHeight;
      break;
    case "bottom-right":
      actualMoveX = moveX + originWidth;
      actualMoveY = moveY + originHeight;
      break;
    case "bottom":
      actualMoveX = originWidth;
      actualMoveY = moveY + originHeight;
      break;
    case "bottom-left":
      actualMoveX = moveX - originWidth;
      actualMoveY = moveY + originHeight;
      break;
    case "left":
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

export const bindDoubleClick = (
  target: HTMLDivElement,
  cb: (e: MouseEvent) => void
) => {
  let clickCount = 0;
  const doubleClickInterval = 300; // 300ms 内双击被认为是双击事件
  const onClick = (e: MouseEvent) => {
    clickCount++;
    if (clickCount === 2) {
      cb(e);
      clickCount = 0;
    } else {
      setTimeout(() => {
        clickCount = 0;
      }, doubleClickInterval);
    }
  };
  target.addEventListener("click", onClick);
  return () => {
    target.removeEventListener("click", onClick);
  };
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
