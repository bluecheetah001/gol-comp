import init, { Viewer } from "./gen/lib.js";

await init();

async function fetch_pattern(name) {
  const response = await fetch(name, { mode: "no-cors" });
  // TODO verify llm
  if (!response.ok) {
    throw new Error(`Failed to load ${name}: HTTP ${response.status}`);
  }
  return await response.text();
}

const width = 1000;
const height = 1000;

const viewer = new Viewer();
try {
  viewer.set_canvas_size(width, height);
  viewer.set_center(0, 0);
  viewer.set_period(undefined);
  viewer.set_step(0);
  viewer.load_base(await fetch_pattern("./pattern/cross.mc"));
  viewer.load_actual(await fetch_pattern("./pattern/and-not+cross.mc"));
  const rgba = viewer.get_rgba();
  console.log(rgba.length);
  // TODO verify llm
  const image = new ImageData(new Uint8ClampedArray(rgba), width, height);
  const canvas = document.querySelector(".viewer");
  canvas.width = width;
  canvas.height = height;
  const context = canvas.getContext("2d");
  context.putImageData(image, 0, 0);
} catch (e) {
  console.error(e);
} finally {
  viewer.free();
}
