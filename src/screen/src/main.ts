import { DrawScreen } from "./draw";
import "./style.css";
import testImg from "./test.png";

const appDom = document.querySelector("#app") as HTMLDivElement;
// const maskDom = document.querySelector(".mask") as HTMLDivElement;
// const imgDom = document.querySelector("#app img") as HTMLImageElement;

function init() {
  // 判断是开发环境还是生产环境
  console.log("Environment mode:", import.meta.env.MODE);
  console.log("Is development:", import.meta.env.DEV);
  console.log("Is production:", import.meta.env.PROD);

  const drawScreen = new DrawScreen(appDom);

  if (import.meta.env.DEV) {
    const imgDom = document.createElement("img");
    imgDom.onload = () => {
      (window as any).drawScreen = drawScreen;
      drawScreen.setImgDom(imgDom);
    };
    imgDom.onerror = (e) => {
      console.error("Image loading failed", e);
    };

    imgDom.setAttribute("src", testImg);
  }

  window.addEventListener('keydown', (e) => {
    if (e.key === 'Escape') {
      (window as any).ipc.postMessage('escape_pressed');
    }
  })
}

init();
