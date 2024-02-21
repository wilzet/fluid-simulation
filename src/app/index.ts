import { Renderer, Resolution, Mode } from "fluid-simulation";
import { isMobile, pixelScaling, randomColor, defaultColor } from "./utils";
import Pointer from "./pointer";

const params = {
    isPaused: false,
    mode: Mode.DYE,
    simResolution: isMobile() ? Resolution.EIGHT : Resolution.TWO,
    dyeResolution: isMobile() ? Resolution.FOUR : Resolution.TWO,
    pointerRadius: 0.2,
    pointerStrength: 10.0,
    viscosity: 1.0,
    dissipation: 1.0,
    curl: 0.25,
    pressure: 0.8,
    color: defaultColor,
    useRandomColor: true,
}
let wasPaused = false;

const canvasId = "canvas";
const canvas = document.getElementById(canvasId) as HTMLCanvasElement;

const renderer = Renderer.create(
    canvasId,
    params.simResolution,
    params.dyeResolution,
);

const pointer = new Pointer([0, 0]);
const pointerColor = new Float32Array(params.color.map((v) => v / 255.0));

const generateColor = () => {
    if (!params.useRandomColor) return;

    const color = randomColor(undefined, undefined, 0.5, 0.9, 0.3, 0.5);
    color.forEach((v, i) => {
        pointerColor[i] = v;
        params.color[i] = v * 255.0;
    });
}

const resizeCanvas = () => {
    canvas.width = pixelScaling(canvas.clientWidth);
    canvas.height = pixelScaling(canvas.clientHeight);

    renderer.resize(params.simResolution, params.dyeResolution);
}

const update = (timestamp: number) => {
    requestAnimationFrame(update);

    const radius = params.pointerRadius * Math.min(canvas.width, canvas.height) * 10.0;

    if (pointer.isMoved) {
        renderer.splat(
            radius,
            pointer.getPosition,
            pointer.getVelocity,
            pointerColor,
        );
    }

    renderer.update(
        params.isPaused,
        timestamp / 1000,
        params.mode,
        params.viscosity,
        params.dissipation,
        -params.curl,
        params.pressure,
    );

    pointer.resetMove();
}

const pointerEvents = () => {
    // DOWN
    canvas.addEventListener("mousedown", (e) => {
        const x = pixelScaling(e.pageX);
        const y = pixelScaling(canvas.clientHeight - e.pageY);
        pointer.down(x, y);
        generateColor();
    });

    canvas.addEventListener("touchstart", (e) => {
        e.preventDefault();
        if (!e.targetTouches[0]) return;

        const x = pixelScaling(e.targetTouches[0].pageX);
        const y = pixelScaling(canvas.clientHeight - e.targetTouches[0].pageY);
        pointer.down(x, y);
        generateColor();
    });

    // MOVE
    canvas.addEventListener("mousemove", (e) => {
        const x = pixelScaling(e.pageX);
        const y = pixelScaling(canvas.clientHeight - e.pageY);
        pointer.update(x, y, params.pointerStrength);
    });

    canvas.addEventListener("touchmove", (e) => {
        e.preventDefault();
        if (!e.targetTouches[0]) return;

        const x = pixelScaling(e.targetTouches[0].pageX);
        const y = pixelScaling(canvas.clientHeight - e.targetTouches[0].pageY);
        pointer.update(x, y, params.pointerStrength);
    });

    // UP
    window.addEventListener("mouseup", () => pointer.up());

    window.addEventListener("touchend", () => pointer.up());
}

const run = () => {
    window.addEventListener("resize", resizeCanvas);

    document.addEventListener("visibilitychange", () => {
        if (document.hidden) {
            params.isPaused = true;
            pointer.up();
        } else if (!wasPaused) {
            params.isPaused = false;
        }
    });

    window.addEventListener("keydown", (e) => {
        if (e.code === "Space") {
            params.isPaused = !params.isPaused;
        }
    });

    pointerEvents();

    resizeCanvas();

    requestAnimationFrame(update);
}

run();