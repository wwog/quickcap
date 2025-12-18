export type EditCanvasMode = "normal" | "edit";

export type TShapeRect = {
  id: string;
  shape: "rect";
  attr: {
    x: number;
    y: number;
    width: number;
    height: number;
  };
  pen: {
    color: string;
    lineWidth: number;
  };
};

export type TShapeCircle = {
  id: string;
  shape: "circle";
  attr: {
    centerX: number;
    centerY: number;
    radiusX: number;
    radiusY: number;
    isCircle: boolean;
    left: number;
    top: number;
    width: number;
    height: number;
  };
  pen: {
    color: string;
    lineWidth: number;
  };
};

export type TShapePath = {
  id: string;
  shape: "path";
  attr: {
    path: Array<{ x: number; y: number }>;
  },
  pen: {
    color: string;
    lineWidth: number;
  };
}

export type TShapeMosaic = {
  id: string;
  shape: "mosaic";
  attr: {
    path: Array<{ x: number; y: number }>;
    radius: number;
  }
}

export type TShapeArrow = {
  id: string;
  shape: "arrow";
  attr: {
    fromX: number;
    fromY: number;
    toX: number;
    toY: number;
  };
  pen: {
    color: string;
    lineWidth: number;
  };
}


export type TShape = TShapeRect | TShapeCircle | TShapePath | TShapeArrow | TShapeMosaic;
