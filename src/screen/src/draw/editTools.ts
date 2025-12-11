import { editToolGap, editToolHeight } from "../const";

export const items = [
  {
    id: "download",
    className: "download",
    content: `<svg width="48" height="48" viewBox="0 0 48 48" fill="none" xmlns="http://www.w3.org/2000/svg">
<rect opacity="0" width="48" height="48" fill="#D8D8D8"/>
<path fill-rule="evenodd" clip-rule="evenodd" d="M24.0416 2C21.8325 2 20.0416 3.79086 20.0416 6V31.8258L12.7749 24.217C11.2251 22.5943 8.71225 22.5943 7.16239 24.217C5.61254 25.8396 5.61254 28.4704 7.16239 30.093L20.897 44.4724C21.2958 44.979 21.8145 45.3869 22.4103 45.6533C22.9221 45.8873 23.4715 46.0029 24.0205 45.9999C24.0275 46 24.0346 46 24.0416 46C25.4652 46 26.7151 45.2564 27.424 44.1363L40.8376 30.093C42.3875 28.4704 42.3875 25.8396 40.8376 24.217C39.2877 22.5943 36.7749 22.5943 35.2251 24.217L28.0416 31.7367V6C28.0416 3.79086 26.2508 2 24.0416 2Z" fill="#A2A8C3"/>
</svg>`,
    role: "download",
  },
  {
    id: "cancel",
    className: "cancel",
    content: `<svg
          width="30"
          height="30"
          viewBox="0 0 30 30"
          fill="red"
          xmlns="http://www.w3.org/2000/svg"
        >
          <path
            fill-rule="evenodd"
            clip-rule="evenodd"
            d="M0.596608 0.596608C1.39209 -0.198869 2.68181 -0.198869 3.47729 0.596608L14.9993 12.1186L26.5227 0.596608C27.3182 -0.198869 28.6079 -0.198869 29.4034 0.596608C30.1989 1.39209 30.1989 2.68181 29.4034 3.47729L17.88 14.9993L29.4034 26.5227C30.1989 27.3182 30.1989 28.6079 29.4034 29.4034C28.6079 30.1989 27.3182 30.1989 26.5227 29.4034L14.9993 17.88L3.47729 29.4034C2.68181 30.1989 1.39209 30.1989 0.596608 29.4034C-0.198869 28.6079 -0.198869 27.3182 0.596608 26.5227L12.1186 14.9993L0.596608 3.47729C-0.198869 2.68181 -0.198869 1.39209 0.596608 0.596608Z"
            fill="red"
          />
        </svg>`,
    role: "cancel",
  },
  {
    id: "finish",
    className: "finish",
    content: `<svg
          width="29"
          height="24"
          viewBox="0 0 29 24"
          fill="none"
          xmlns="http://www.w3.org/2000/svg"
        >
          <path
            d="M2 14.3611L9.72055 22L26.4444 2"
            stroke="#00C6DB"
            stroke-width="4"
            stroke-linecap="round"
            stroke-linejoin="round"
          />
        </svg>`,
    role: "finish",
  },
] as const;

// 从 items 数组中提取所有可能的 role 值
type ItemRole = (typeof items)[number]["role"];

export class EditTools {
  dom: HTMLDivElement;
  itemListeners: Map<string, () => void> = new Map();
  constructor(parent?: HTMLElement) {
    this.dom = document.createElement("div");
    this.dom.classList.add("edit-tool");
    this.dom.style.visibility = "hidden";
    const p = parent || document.body;
    if (p) {
      p.appendChild(this.dom);
    }

    this.initItems();
    this.initListeners();
  }

  private initItems = () => {
    items.forEach((item) => {
      const itemDom = document.createElement("div");
      itemDom.classList.add("edit-tool-item", item.className);
      itemDom.innerHTML = item.content || "";
      itemDom.dataset.role = item.role || "";
      this.dom.appendChild(itemDom);
    });
  };

  private initListeners = () => {
    this.dom.addEventListener("click", (e) => {
      let target = e.target as HTMLElement;
      while (target && !target.classList.contains("edit-tool-item")) {
        target = target.parentElement as HTMLElement;
        if (target === this.dom) break;
      }
      if (!target) return;
      const role = target.dataset.role || "";
      console.log("🚀 ~ EditTools ~ role:", role);
      const listener = this.itemListeners.get(role);
      console.log("🚀 ~ EditTools ~ listener:", listener);
      if (listener) {
        listener();
      }
    });
  };

  addListener = (items: { role: ItemRole; listener: () => void }[]) => {
    items.forEach((item) => {
      this.itemListeners.set(item.role, item.listener);
    });
  };

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
    if (!show) {
      this.dom.style.visibility = "hidden";
      return;
    }
    if (this.dom) {
      this.dom.style.visibility = "visible";
      this.dom.style.left = `${x + width - this.dom.clientWidth}px`;
      const maxY = window.innerHeight;
      // const maxX = window.innerWidth;

      // under the selection area
      if (y + height + editToolHeight + editToolGap <= maxY) {
        this.dom.style.top = `${y + height + editToolGap}px`;
        return;
      }
      // above the selection area
      if (y - editToolHeight - editToolGap >= 0) {
        this.dom.style.top = `${y - editToolHeight - editToolGap}px`;
        return;
      }

      // inner the selection area
      this.dom.style.top = `${y + height - editToolHeight - editToolGap}px`;
    }
  };
}
