import { getScreenImageData } from "../utils";

export async function onClickFinish({ ctx, rect }: {
  ctx: CanvasRenderingContext2D;
  rect: {
    x: number;
    y: number;
    width: number;
    height: number;
  }
}) {
  const { x, y, width, height } = rect;
  const screenImageData = await getScreenImageData([x, y, width, height]);
  console.log("🚀 ~ onClickFinish ~ screenImageData:", screenImageData);
}