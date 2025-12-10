import { editToolGap, editToolHeight } from "./const";

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
