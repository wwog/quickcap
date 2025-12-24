// let moved = false;
// let start = false;
// const cbArr: ((e: MouseEvent) => void)[] = [];
// function initMouseEvent() {
//   const onMouseDown = (e: MouseEvent) => {
//     start = true;
//     moved = false;
//   };
//   const onMouseUp = (e: MouseEvent) => {
//     if (start && !moved) {
//       cbArr.forEach((cb) => cb(e));
//     }
//     start = false;
//     moved = false;
//   };
//   const onMouseLeave = (e: MouseEvent) => {
//     if (start) {
//       moved = true;
//     }
//   };
//   const onMouseMove = (e: MouseEvent) => {
//     if (start) {
//       moved = true;
//     }
//   };
//   document.addEventListener("mousedown", onMouseDown);
//   document.addEventListener("mousemove", onMouseMove);
//   document.addEventListener("mouseup", onMouseUp);
//   document.addEventListener("mouseleave", onMouseLeave);
// }

// initMouseEvent();

export const bindClick = (
  target: HTMLDivElement,
  cb: (e: MouseEvent) => void
) => {
  let moved = false;
  let start = false;
  const onMouseDown = (e: MouseEvent) => {
    if (e.target === target || target.contains(e.target as Node)) {
      start = true;
      moved = false;
    }
  };
  const onMouseUp = (e: MouseEvent) => {
    if (start && !moved) {
      cb(e);
    }
    start = false;
    moved = false;
  };
  const onMouseLeave = () => {
    if (start) {
      moved = true;
    }
  };
  const onMouseMove = () => {
    if (start) {
      moved = true;
    }
  };
  document.addEventListener("mousedown", onMouseDown);
  document.addEventListener("mousemove", onMouseMove);
  document.addEventListener("mouseup", onMouseUp);
  document.addEventListener("mouseleave", onMouseLeave);
  return () => {
    document.removeEventListener("mousedown", onMouseDown);
    document.removeEventListener("mousemove", onMouseMove);
    document.removeEventListener("mouseup", onMouseUp);
    document.removeEventListener("mouseleave", onMouseLeave);
  };
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
  const unbindClick = bindClick(target, onClick);
  return unbindClick;
  // target.addEventListener("click", onClick);
  // return () => {
  //   target.removeEventListener("click", onClick);
  // };
};
