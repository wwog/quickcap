import { DrawScreen } from "./draw";
import "./style.css";
// import testImg from "./test.png";
import { exitApp, getScreenImageData } from "./utils";

const appDom = document.querySelector("#app") as HTMLDivElement;
// const maskDom = document.querySelector(".mask") as HTMLDivElement;
// const imgDom = document.querySelector("#app img") as HTMLImageElement;

function init() {
  // 判断是开发环境还是生产环境
  console.log("Environment mode:", import.meta.env.MODE);
  console.log("Is development:", import.meta.env.DEV);
  console.log("Is production:", import.meta.env.PROD);

  const drawScreen = new DrawScreen(appDom);
  console.log("🚀 ~ init ~ drawScreen:", drawScreen);

  // if (import.meta.env.DEV) {
  //   const imgDom = document.createElement("img");
  //   imgDom.onload = () => {
  //     (window as any).drawScreen = drawScreen;
  //     drawScreen.setImgDom(imgDom);
  //   };
  //   imgDom.onerror = (e) => {
  //     console.error("Image loading failed", e);
  //   };

  //   imgDom.setAttribute("src", testImg);
  // }

  /* const setImgDom = () => {
    const imgDom = document.createElement("img");
    imgDom.onload = () => {
      (window as any).drawScreen = drawScreen;
      drawScreen.setImgDom(imgDom);
    };
    imgDom.onerror = (e) => {
      console.error("Image loading failed", e);
    };
    imgDom.setAttribute("src", testImg);
  };
  (window as any).setImgDom = setImgDom;

  setImgDom(); */

  getScreenImageData()
    .then((imgData) => {
      console.log("🚀 ~ init ~ imgData:", imgData);
      drawScreen.putImageData(imgData);
    })
    .catch((err) => console.error(err));

  window.addEventListener("keydown", (e) => {
    console.log("🚀 ~ init ~ e:", e.key, e.keyCode);
    if (e.key === "Escape") {
      alert("Escape pressed");
      exitApp();
    }
  });
  appDom.addEventListener("keydown", (e) => {
    console.log("🚀 ~ appDom init ~ e:", e.key);
    if (e.key === "Escape") {
      alert("Escape pressed");
      exitApp();
    }
  });
}

init();
