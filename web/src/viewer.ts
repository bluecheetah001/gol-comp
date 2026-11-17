import { Viewer } from "../gen/lib";

interface ViewerState {
  // display objects
  viewer: Viewer;
  canvas: HTMLCanvasElement;
  context: CanvasRenderingContext2D;
  width: number;
  height: number;

  // logical state
  step: number;
}

export function start_viewer(canvas: HTMLCanvasElement) {
  create_viewer(canvas)
    .then(start_viewer_animation)
    .catch((e) => console.warn(e));
}
async function create_viewer(canvas: HTMLCanvasElement): Promise<ViewerState> {
  const context = canvas.getContext("2d");
  if (context == null) {
    throw new Error("Failed to get 2d context");
  }
  // TODO set size via CSS and resize canvas as needed
  const width = Math.round(canvas.width);
  const height = Math.round(canvas.height);
  const [base, pattern] = await Promise.all([
    fetch_pattern_attribute(canvas, "data-base"),
    fetch_pattern_attribute(canvas, "data-pattern"),
  ]);
  const period = get_opt_int_attribute(canvas, "data-period");
  const step = 0;

  const viewer = new Viewer();
  viewer.load_base(base);
  viewer.load_actual(pattern);
  viewer.set_period(period);
  viewer.set_canvas_size(width, height);
  viewer.set_step(step);
  return {
    viewer,
    canvas,
    context,
    width,
    height,
    step,
  };
}

function start_viewer_animation(state: ViewerState) {
  const frame = () => {
    try {
      update_viewer(state);
      requestAnimationFrame(frame);
    } catch (e) {
      console.warn(e);
    }
  };
  requestAnimationFrame(frame);
}
function update_viewer(state: ViewerState) {
  state.step += 1;
  state.viewer.set_step(state.step);
  const rgba = state.viewer.get_rgba();
  const image = new ImageData(new Uint8ClampedArray(rgba), state.width, state.height);
  state.context.putImageData(image, 0, 0);
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
