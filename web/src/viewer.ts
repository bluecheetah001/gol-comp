import { Viewer } from "../gen/lib";

export function start_viewer(canvas: HTMLCanvasElement) {
  let viewer = new Viewer();
  const on_error = (e: unknown) => {
    console.warn(e);
    viewer.free();
  };
  setup_viewer(viewer, canvas)
    .then((on_frame) => animate(on_frame, on_error))
    .catch(on_error);
}
async function setup_viewer(viewer: Viewer, canvas: HTMLCanvasElement) {
  const context = canvas.getContext("2d");
  if (context == null) {
    throw new Error("Failed to get 2d context");
  }
  const width = canvas.width;
  const height = canvas.height;
  const [base, pattern] = await Promise.all([
    fetch_pattern_attribute(canvas, "data-base"),
    fetch_pattern_attribute(canvas, "data-pattern"),
  ]);
  const period = get_opt_int_attribute(canvas, "data-period");
  viewer.load_base(base);
  viewer.load_actual(pattern);
  viewer.set_period(period);
  viewer.set_canvas_size(width, height);

  // per frame logic
  let step = 0;
  return () => {
    step += 1;
    viewer.set_step(step);
    const rgba = viewer.get_rgba();
    const image = new ImageData(new Uint8ClampedArray(rgba), width, height);
    context.putImageData(image, 0, 0);
  };
}

async function fetch_pattern_attribute(canvas: HTMLCanvasElement, attribute: string): Promise<string> {
  const url = canvas.getAttribute(attribute);
  if (url == null) {
    throw new Error(`Missing ${attribute} attribute`);
  }
  const response = await fetch(url, { mode: "no-cors" });
  if (!response.ok) {
    throw new Error(`Failed to load ${attribute}=${url}: HTTP ${response.status}`);
  }
  return await response.text();
}
function get_opt_int_attribute(canvas: HTMLCanvasElement, attribute: string): number | undefined {
  const raw = canvas.getAttribute(attribute);
  if (raw == null) {
    return undefined;
  }
  if (/-?[0-9]+/.exec(raw) != null) {
    return Number(raw);
  } else {
    throw new Error(`Invalid ${attribute}=${raw}, expected integer`);
  }
}
function animate(callback: () => void, on_error: (e: unknown) => void) {
  const wrapped = () => {
    try {
      callback();
      requestAnimationFrame(wrapped);
    } catch (e) {
      on_error(e);
    }
  };
  requestAnimationFrame(wrapped);
}
