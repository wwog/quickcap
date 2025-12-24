import { editToolGap, editToolHeight } from "../const";

export const items = [
  {
    id: "rect",
    className: "box-select rect",
    content: `<div class="wrap"><svg width="19" height="19" viewBox="0 0 19 19" fill="none" xmlns="http://www.w3.org/2000/svg"><rect x="0.75" y="0.75" width="16.8" height="16.8" rx="1.2" stroke="var(--active-color)" stroke-width="1.5"/></svg></div>`,
    role: "edit",
    shape: "rect",
    group: "edit",
  },
  {
    id: "circle",
    className: "box-select circle",
    content: `<div class="wrap"><svg width="19" height="19" viewBox="0 0 19 19" fill="none" xmlns="http://www.w3.org/2000/svg"><rect x="0.75" y="0.75" width="16.8" height="16.8" rx="8.4" stroke="var(--active-color)" stroke-width="1.5"/></svg></div>`,
    role: "edit",
    shape: "circle",
    group: "edit",
  },
  {
    id: "arrow",
    className: "box-select arrow",
    content: `<div class="wrap"><svg width="24" height="24" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg"><path d="M4.7998 20.0732L20.0733 4.79974" stroke="var(--active-color)" stroke-width="1.5" stroke-linecap="round"/><path d="M20.4082 12.9833L20.4082 4.5752L12.0001 4.57519" stroke="var(--active-color)" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/></svg></div>`,
    role: "edit",
    shape: "arrow",
    group: "edit",
  },
  {
    id: "path",
    className: "box-select path",
    content: `<div class="wrap"><svg width="24" height="24" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg"><path d="M15.1815 3.68472C15.8482 3.01806 16.1815 2.68472 16.5957 2.68472C17.0099 2.68472 17.3433 3.01805 18.0099 3.68472L20.3137 5.9885C20.9804 6.65516 21.3137 6.9885 21.3137 7.40271C21.3137 7.81692 20.9804 8.15026 20.3137 8.81692L9.00331 20.1273C8.72319 20.4074 8.58313 20.5475 8.40558 20.6234C8.22803 20.6993 8.03001 20.7037 7.63396 20.7126L3.60427 20.803C3.40788 20.8074 3.30968 20.8096 3.24923 20.7492C3.18879 20.6887 3.19099 20.5905 3.1954 20.3941L3.28582 16.3645C3.29471 15.9684 3.29915 15.7704 3.37504 15.5928C3.45093 15.4153 3.59098 15.2752 3.8711 14.9951L15.1815 3.68472Z" stroke="var(--active-color)" stroke-width="1.5"/><line x1="13.6153" y1="4.63373" x2="19.4763" y2="10.4948" stroke="var(--active-color)" stroke-width="1.5"/></svg></div>`,
    role: "edit",
    shape: "path",
    group: "edit",
  },
  {
    id: "mosaic",
    className: "box-select mosaic",
    content: `<div class="wrap"><svg width="24" height="24" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg"><rect x="3.59961" y="3.59961" width="16.8" height="16.8" rx="1" stroke="var(--active-color)" stroke-width="1.5"/><rect x="10.2002" y="10.2002" width="3.6" height="3.6" rx="0.2" fill="var(--active-color)"/><rect x="13.7998" y="6.59961" width="3.6" height="3.6" rx="0.2" fill="var(--active-color)"/><rect x="6.59961" y="6.59961" width="3.6" height="3.6" rx="0.2" fill="var(--active-color)"/><rect x="6.59961" y="13.7998" width="3.6" height="3.6" rx="0.2" fill="var(--active-color)"/><rect x="13.7998" y="13.7998" width="3.6" height="3.6" rx="0.2" fill="var(--active-color)"/></svg></div>`,
    role: "edit",
    shape: "mosaic",
    group: "edit",
  },
  {
    id: "undo",
    className: "undo",
    content: `<div class="wrap"><svg class="normal" style="display: block;" width="24" height="24" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg"><path d="M10.3245 19.7779H15.3382C18.4533 19.7779 20.9786 17.2526 20.9786 14.1375C20.9786 11.0224 18.4533 8.49707 15.3382 8.49707H3.43066" stroke="#DBDEE5" stroke-width="1.5" stroke-linecap="round"/><path d="M7.85079 3.27421L2.62793 8.49707L7.85079 13.7199" stroke="#DBDEE5" stroke-width="1.2" stroke-linecap="round" stroke-linejoin="round"/></svg><svg class="active" style="display: none;" width="24" height="24" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg"><path d="M10.3245 19.7779H15.3382C18.4533 19.7779 20.9786 17.2526 20.9786 14.1375C20.9786 11.0224 18.4533 8.49707 15.3382 8.49707H3.43066" stroke="#0D1324" stroke-width="1.5" stroke-linecap="round"/><path d="M7.85079 3.27421L2.62793 8.49707L7.85079 13.7199" stroke="#0D1324" stroke-width="1.2" stroke-linecap="round" stroke-linejoin="round"/></svg></div>`,
    role: "undo",
    group: "operation",
  },
  {
    id: "download",
    className: "download",
    content: `<svg width="24" height="24" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg"><path d="M12.75 2.71387C12.75 2.29965 12.4142 1.96387 12 1.96387C11.5858 1.96387 11.25 2.29965 11.25 2.71387L12 2.71387L12.75 2.71387ZM11.4697 17.3372C11.7626 17.6301 12.2374 17.6301 12.5303 17.3372L17.3033 12.5642C17.5962 12.2713 17.5962 11.7964 17.3033 11.5035C17.0104 11.2106 16.5355 11.2106 16.2426 11.5035L12 15.7462L7.75736 11.5035C7.46447 11.2106 6.98959 11.2106 6.6967 11.5035C6.40381 11.7964 6.40381 12.2713 6.6967 12.5642L11.4697 17.3372ZM12 2.71387L11.25 2.71387L11.25 16.8068L12 16.8068L12.75 16.8068L12.75 2.71387L12 2.71387Z" fill="#0D1324"/><line x1="2.5498" y1="20.5361" x2="21.4498" y2="20.5361" stroke="#0D1324" stroke-width="1.5" stroke-linecap="round"/></svg>`,
    role: "download",
    group: "operation",
  },
  {
    id: "cancel",
    className: "cancel",
    content: `<svg width="24" height="24" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg"><path d="M19.4821 4.46875L4.41895 19.5319" stroke="#FE4343" stroke-width="1.5" stroke-linecap="round"/><path d="M19.5807 19.5319L4.51758 4.46875" stroke="#FE4343" stroke-width="1.5" stroke-linecap="round"/></svg>`,
    role: "cancel",
    group: "operation",
  },
  {
    id: "finish",
    className: "finish",
    content: `<svg width="24" height="24" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg"><path d="M20.7379 4.92676L10.9616 17.6765C10.2749 18.572 9.93156 19.0198 9.44463 19.0692C8.95771 19.1187 8.53133 18.7491 7.67858 18.01L3.2627 14.1824" stroke="#32C872" stroke-width="1.5" stroke-linecap="round"/></svg>`,
    role: "finish",
    group: "operation",
  },
] as const;

// 从 items 数组中提取所有可能的 role 值
type ItemRole = (typeof items)[number]["role"];

const HEIGHT = 40;

export class EditTools {
  dom: HTMLDivElement;
  itemListeners: Map<string, (str?: string) => void> = new Map();

  private _active: string = "";

  private _undoActive = false;

  get active() {
    return this._active;
  }

  set active(shape: string) {
    this.dom
      .querySelector(".edit-tool-item.active")
      ?.classList.remove("active");
    if (shape) {
      this.dom
        .querySelector(`.edit-tool-item[data-shape="${shape}"]`)
        ?.classList.add("active");
    }
    this._active = shape;
  }

  get undoActive() {
    return this._undoActive;
  }

  set undoActive(active: boolean) {
    if (this._undoActive === active) return;
    this._undoActive = active;
    if (active) {
      (
        this.dom.querySelector(".edit-tool-item.undo .active") as HTMLDivElement
      ).style.display = "block";
      (
        this.dom.querySelector(".edit-tool-item.undo .normal") as HTMLDivElement
      ).style.display = "none";
    } else {
      (
        this.dom.querySelector(".edit-tool-item.undo .normal") as HTMLDivElement
      ).style.display = "block";
      (
        this.dom.querySelector(".edit-tool-item.undo .active") as HTMLDivElement
      ).style.display = "none";
    }
  }

  constructor(parent?: HTMLElement) {
    this.dom = document.createElement("div");
    this.dom.classList.add("edit-tool");
    this.dom.style.visibility = "hidden";
    this.dom.style.height = `${HEIGHT}px`;
    const p = parent || document.body;
    if (p) {
      p.appendChild(this.dom);
    }

    this.initItems();
    this.initListeners();
  }

  private initItems = () => {
    const groupDomMap: {
      [key: string]: HTMLDivElement;
    } = {};
    items.forEach((item) => {
      const group = item.group || "";
      if (!groupDomMap[group]) {
        groupDomMap[group] = document.createElement("div");
        groupDomMap[group].classList.add("edit-tool-group");
        groupDomMap[group].dataset.group = group;
        this.dom.appendChild(groupDomMap[group]);
      }
      const itemDom = document.createElement("div");
      itemDom.classList.add("edit-tool-item", ...item.className.split(" "));
      itemDom.innerHTML = item.content || "";
      itemDom.dataset.role = item.role || "";
      itemDom.dataset.shape = (item as any).shape || "";
      groupDomMap[group].appendChild(itemDom);
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
      const shape = target.dataset.shape || undefined;
      const listener = this.itemListeners.get(role);
      if (listener) {
        listener(shape);
      }
    });
  };

  addListener = (
    items: { role: ItemRole; listener: (shape?: string) => void }[]
  ) => {
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
