import { DPR, sizeDisplayGap, sizeDisplayHeight } from "../const";

export class SizeDisplay {
  private dom: HTMLDivElement;
  constructor(parentDom: HTMLDivElement) {
    this.dom = document.createElement("div");
    this.dom.classList.add("size-display");
    parentDom.appendChild(this.dom);

  }

  render = (
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
    if (!show || (!height && !width)) {
      this.dom.style.visibility = "hidden";
      return;
    }
    this.dom.style.visibility = "visible";
    this.dom.style.left = `${x}px`;
    if (y - sizeDisplayHeight - sizeDisplayGap < 0) {
      this.dom.style.top = `${y + sizeDisplayGap}px`;
    } else {
      this.dom.style.top = `${y - sizeDisplayHeight - sizeDisplayGap}px`;
    }
    this.dom.innerText = `${width * DPR} x ${height * DPR}`;
  };
}
