import init from "../gen/lib.js";
import { start_viewer } from "./viewer.js";

await init();

for (const canvas of document.querySelectorAll<HTMLCanvasElement>("canvas.viewer")) {
  start_viewer(canvas);
}
