import * as dat from "dat.gui";
import { Renderer, Resolution, Mode } from "fluid-simulation";
import { isMobile, pixelScaling, randomColor, defaultColor } from "./utils";
import Pointer from "./pointer";

const params = {
    isPaused: false,
    mode: Mode.DYE,
    simResolution: isMobile() ? Resolution.EIGHT : Resolution.FOUR,
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

const createGUI = () => {
    const gui = new dat.GUI({ closeOnTop: true });

    const visualsFolder = gui.addFolder("Visuals");
    visualsFolder.add(
        params,
        "mode",
        {
            "Dye": Mode.DYE,
            "Velocity": Mode.VELOCITY,
            "Pressure": Mode.PRESSURE,
        },
    )
        .name("Mode");
    visualsFolder.add(
        params,
        "dyeResolution",
        {
            "Ultra+": Resolution.ONE,
            "Ultra": Resolution.TWO,
            "High": Resolution.FOUR,
            "Medium": Resolution.EIGHT,
            "Low": Resolution.SIXTEEN,
        },
    )
        .name("Quality")
        .onFinishChange(resizeCanvas);
    visualsFolder.open();

    const simulationFolder = gui.addFolder("Simulation");
    simulationFolder.add(
        params,
        "simResolution",
        {
            "Ultra+": Resolution.ONE,
            "Ultra": Resolution.TWO,
            "High": Resolution.FOUR,
            "Medium": Resolution.EIGHT,
            "Low": Resolution.SIXTEEN,
        },
    )
        .name("Simulation quality")
        .onFinishChange(resizeCanvas);
    simulationFolder.add(params, "viscosity", 0.0, 5.0, 0.01).name("Viscosity");
    simulationFolder.add(params, "dissipation", 0.0, 5.0, 0.01).name("Dye diffusion");
    simulationFolder.add(params, "curl", 0.0, 1.0, 0.01).name("Vorticity amount");
    simulationFolder.add(params, "pressure", 0.0, 1.0, 0.01).name("Pressure");
    simulationFolder.open();

    const colorFolder = gui.addFolder("Color");
    colorFolder.addColor(params, "color").name("Color").onFinishChange((value: number[]) => {
        value.forEach((v, i) => pointerColor[i] = v / 255.0);
        params.useRandomColor = false;
    }).listen();
    colorFolder.add(params, "useRandomColor").name("Random color").listen();
    
    gui.add(params, "pointerRadius", 0.01, 1.0, 0.01).name("Radius");
    gui.add(params, "pointerStrength", 0.5, 100.0, 0.01).name("Strength");
    gui.add(params, "isPaused").name("Pause").onChange(() => wasPaused = !wasPaused).listen();

    const gitHub = gui.add({ fun: () => window.open("https://github.com/wilzet/fluid-simulation") }, "fun").name("GitHub");
    const list = gitHub.domElement.parentElement?.parentElement;
    if (list) list.className += " link";
    
    const icon = document.createElement("div");
    list?.appendChild(icon);
    icon.className = "github";

    if (isMobile()) gui.close();
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

    createGUI();

    requestAnimationFrame(update);
}

run();
