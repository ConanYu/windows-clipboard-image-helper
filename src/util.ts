import {CSSProperties} from "react";

export function DateToString(date: Date) {
  const padStart = (x: number) => {
    return x.toString().padStart(2, '0');
  };
  return `${date.getFullYear()}/${padStart(date.getMonth())}/${padStart(date.getDate())} ${padStart(date.getHours())}:${padStart(date.getMinutes())}:${padStart(date.getSeconds())}`;
}

export function CalcImagePaddleStyle(blockWidth: number, blockHeight: number, imageWidth: number, imageHeight: number): CSSProperties {
  if (imageWidth <= 0 || imageHeight <= 0) {
    return {};
  }
  const blockRate = blockWidth / blockHeight;
  const imageRate = imageWidth / imageHeight;
  const paddingUpDown = (() => {
    if (imageRate <= blockRate) {
      return 0;
    }
    return (blockHeight - 1 / imageRate * blockWidth) / 2;
  })();
  const paddingLeftRight = (() => {
    if (imageRate >= blockRate) {
      return 0;
    }
    return (blockWidth - imageRate * blockHeight) / 2;
  })();
  return {
    paddingLeft: paddingLeftRight,
    paddingRight: paddingLeftRight,
    paddingTop: paddingUpDown,
    paddingBottom: paddingUpDown,
  };
}
