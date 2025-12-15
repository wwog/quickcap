import { DrawScreen } from "./draw";
import "./style.css";
import testImg from "./test.png";

const appDom = document.querySelector("#app") as HTMLDivElement;
// const maskDom = document.querySelector(".mask") as HTMLDivElement;
// const imgDom = document.querySelector("#app img") as HTMLImageElement;

function init() {
  const imgDom = document.createElement("img");
  imgDom.onload = () => {
    const drawScreen = new DrawScreen(appDom, imgDom);
    (window as any).drawScreen = drawScreen;
  };
  imgDom.onerror = (e) => {
    console.error("Image loading failed", e);
  };

  imgDom.setAttribute("src", testImg);
}

init();
