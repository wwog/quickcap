import { DrawScreen } from "./draw";
import "./style.css";
import { exitApp, getScreenImageData } from "./utils";

const appDom = document.querySelector("#app") as HTMLDivElement;

function init() {
  // 判断是开发环境还是生产环境
  console.log("Environment mode:", import.meta.env.MODE);
  console.log("Is development:", import.meta.env.DEV);
  console.log("Is production:", import.meta.env.PROD);

  const drawScreen = new DrawScreen(appDom);
  console.log("🚀 ~ init ~ drawScreen:", drawScreen);
  (window as any).drawScreen = drawScreen;

  getScreenImageData()
    .then((imgData) => {
      drawScreen.putImageData(imgData);
    })
    .catch((err) => console.error(err));

  window.addEventListener("keydown", (e) => {
    if (e.key === "Escape") {
      exitApp();
    }
  });
}

init();
