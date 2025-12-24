import { resizeHandles } from "../const";

export class ResizeAssist {
  private dom: HTMLDivElement;

  constructor() {
    this.dom = document.createElement("div");
    this.dom.classList.add("resize-assist");
    this.dom.style.display = "none";
    resizeHandles.forEach((handleName) => {
      const handleDom = document.createElement("div");
      handleDom.classList.add("resize-assist-handle", handleName);
      handleDom.dataset.role = handleName;
      this.dom.appendChild(handleDom);
    });
    document.body.appendChild(this.dom);
  }

  show() {
    this.dom.style.display = "block";
  }

  hide() {
    this.dom.style.display = "none";
  }

  setPosition({ top, left, width, height }: DOMRect) {
    this.dom.style.top = `${top}px`;
    this.dom.style.left = `${left}px`;
    this.dom.style.width = `${width}px`;
    this.dom.style.height = `${height}px`;
  }
}
