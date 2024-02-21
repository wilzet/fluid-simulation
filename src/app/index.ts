import { Renderer, Resolution } from "fluid-simulation";

const CANVAS_ID = "canvas";
const resolution = Resolution.TWO;

const canvas = document.getElementById(CANVAS_ID) as HTMLCanvasElement;
const renderer = Renderer.create(CANVAS_ID, resolution, resolution);
let mousePosition = new Float32Array([0, 0]);
let mouseVelocity = new Float32Array([0, 0]);

const update = (timestamp: number) => {
    requestAnimationFrame(update);

    renderer.update(
        timestamp / 1000,
        mousePosition,
        mouseVelocity,
        300,
        1.0,
        1.0,
        0.5,
        0.8,
    );
}

const resizeCanvas = () => {
    canvas.width = window.innerWidth;
    canvas.height = window.innerHeight;

    renderer.resize(resolution, resolution);
}

const run = () => {
    window.addEventListener('resize', resizeCanvas);

    canvas.addEventListener('mousemove', (e) => {
        const x = e.clientX;
        const y = (canvas.clientHeight - e.clientY);
        mouseVelocity[0] = (x - (mousePosition[0] ?? 0)) * 1000.0;
        mouseVelocity[1] = (y - (mousePosition[1] ?? 0)) * 1000.0;
        mousePosition[0] = x;
        mousePosition[1] = y;
    });

    resizeCanvas();

    requestAnimationFrame(update);
}

run();