import * as dat from "dat.gui";
import { Renderer, Resolution, Mode } from "fluid-simulation";
import { isMobile, pixelScaling, randomColor, defaultBlueColor, defaultRedColor } from "./utils";
import Pointer from "./pointer";

enum Configuration {
    NONE,
    SPELLS,
    SPIN,
};

const params = {
    isPaused: false,
    mode: Mode.DYE,
    dyeResolution: Resolution.TWO,
    simResolution: isMobile() ? Resolution.EIGHT : Resolution.FOUR,
    pointerRadius: isMobile() ? 0.4 : 0.2,
    pointerStrength: 10.0,
    iterations: 20,
    viscosity: 0.5,
    dissipation: 2.0,
    curl: 0.25,
    pressure: 0.8,
    color: defaultBlueColor,
    useRandomColor: true,
    config: Configuration.NONE,
};
let wasPaused = false;

const config = {
    position: new Float32Array([0.0, 0.0]),
    velocity: new Float32Array([0.0, 0.0]),
    color: new Float32Array([0.0, 0.0, 0.0]),
    lColor: defaultRedColor,
    lRadius: 0.2,
    lStrength: 10.0,
    lXOffset: 0.05,
    lYOffset: 0.0,
    rColor: defaultBlueColor.slice(),
    rRadius: 0.2,
    rStrength: 10.0,
    rXOffset: 0.05,
    rYOffset: 0.0,
};
const pointer = new Pointer([0, 0]);
const pointerColor = new Float32Array(params.color.map((v) => v / 255.0));

const canvasId = "canvas";
const canvas = document.getElementById(canvasId) as HTMLCanvasElement;

const renderer = Renderer.create(
    canvasId,
    params.simResolution,
    params.dyeResolution,
);

const generateColor = () => {
    if (!params.useRandomColor) return;

    randomColor(undefined, undefined, 0.5, 0.9, 0.3, 0.5).forEach((v, i) => {
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
    simulationFolder.add(params, "viscosity", 0.0, 5.0, 0.01).name("Viscosity");
    simulationFolder.add(params, "dissipation", 0.0, 5.0, 0.01).name("Dye diffusion");
    simulationFolder.add(params, "curl", 0.0, 2.0, 0.01).name("Vorticity amount");
    simulationFolder.open();

    const advancedFolder = simulationFolder.addFolder("Advanced");
    advancedFolder.add(
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
        .onFinishChange((value: number) => {
            params.iterations = isMobile() ? 20 : value <= 3 ? value <= 2 ? value <= 1 ? 50 : 40 : 30 : 20;
            resizeCanvas();
        });
    advancedFolder.add(params, "pressure", 0.0, 1.0, 0.01).name("Pressure");
    advancedFolder.add(params, "iterations", 10, isMobile() ? 50 : 80, 1).name("Solver iterations").listen();
    
    const configurationFolder = gui.addFolder("Configuration");
    let settingsFolder: dat.GUI | undefined;
    configurationFolder.add(
        params,
        "config",
        {
            "None": Configuration.NONE,
            "Spells": Configuration.SPELLS,
            "Spin": Configuration.SPIN,
        },
    )
        .name("Configuration")
        .onFinishChange((value: number) => {
            if (settingsFolder) {
                configurationFolder.removeFolder(settingsFolder);
                settingsFolder = undefined;
            }

            if (value == Configuration.SPELLS) {
                settingsFolder = configurationFolder.addFolder("Spell Settings");
                
                const leftFolder = settingsFolder.addFolder("Left");
                leftFolder.addColor(config, "lColor").name("Color");
                config.lRadius = Math.min(config.lRadius, 2.0);
                leftFolder.add(config, "lRadius", 0.01, 2.0, 0.01).name("Radius");
                leftFolder.add(config, "lStrength", 0.0, 100.0, 0.01).name("Strength");
                config.lXOffset = Math.max(config.lXOffset, 0.0);
                leftFolder.add(config, "lXOffset", 0.0, 1.0, 0.01).name("X");
                leftFolder.add(config, "lYOffset", -1.0, 1.0, 0.01).name("Y");

                const rightFolder = settingsFolder.addFolder("Right");
                rightFolder.addColor(config, "rColor").name("Color");
                config.rRadius = Math.max(Math.min(config.rRadius, 2.0), 0.01);
                rightFolder.add(config, "rRadius", 0.01, 2.0, 0.01).name("Radius");
                config.rStrength = Math.max(config.rStrength, 0.0);
                rightFolder.add(config, "rStrength", 0.0, 100.0, 0.01).name("Strength");
                rightFolder.add(config, "rXOffset", 0.0, 1.0, 0.01).name("X");
                rightFolder.add(config, "rYOffset", -1.0, 1.0, 0.01).name("Y");

                settingsFolder.open();
            } else if (value == Configuration.SPIN) {
                settingsFolder = configurationFolder.addFolder("Spin Settings");
                settingsFolder.add(config, "rRadius", -2 * Math.PI, 2 * Math.PI, 0.01).name("Rotation speed");
                config.rStrength = Math.min(config.rStrength, Math.PI);
                settingsFolder.add(config, "rStrength", -Math.PI, Math.PI, 0.01).name("Angular offset");
                settingsFolder.addColor(config, "lColor").name("Color");
                settingsFolder.add(config, "lRadius", 0.01, 3.0, 0.01).name("Radius");
                settingsFolder.add(config, "lStrength", 0.0, 100.0, 0.01).name("Strength");
                // Swap X and Y because of initial offset
                settingsFolder.add(config, "lYOffset", -1.0, 1.0, 0.01).name("X");
                settingsFolder.add(config, "lXOffset", -1.0, 1.0, 0.01).name("Y");

                settingsFolder.open();
            }
        });

    const pointerFolder = gui.addFolder("Pointer");
    pointerFolder.addColor(params, "color").name("Color").onFinishChange((value: number[]) => {
        value.forEach((v, i) => pointerColor[i] = v / 255.0);
        params.useRandomColor = false;
    }).listen();
    pointerFolder.add(params, "useRandomColor").name("Random color").listen();
    pointerFolder.add(params, "pointerRadius", 0.01, 1.0, 0.01).name("Radius");
    pointerFolder.add(params, "pointerStrength", 0.5, 100.0, 0.01).name("Strength");
    pointerFolder.open();

    gui.add(params, "isPaused").name("Pause").onChange(() => wasPaused = !wasPaused).listen();

    const gitHub = gui.add({ fun: () => window.open("https://github.com/wilzet/fluid-simulation") }, "fun").name("GitHub");
    const icon = document.createElement("div");
    icon.className = "github";
    const list = gitHub.domElement.parentElement?.parentElement;
    if (list) list.className += " link";
    list?.appendChild(icon);

    if (isMobile()) gui.close();
}

const spellConfig = (radius: number) => {
    const width = canvas.width;
    const halfHeight = canvas.height * 0.5;

    config.position[0] = width * config.lXOffset;
    config.position[1] = (1 + config.lYOffset) * halfHeight;
    config.velocity[0] = 10.0 * config.lStrength;
    config.velocity[1] = 0.0;
    config.lColor.forEach((v, i) => config.color[i] = v / 255.0);
    renderer.splat(
        radius * config.lRadius,
        config.position,
        config.velocity,
        config.color,
    );

    config.position[0] = (1.0 - config.rXOffset) * width;
    config.position[1] = (1 + config.rYOffset) * halfHeight;
    config.velocity[0] = -10.0 * config.rStrength;
    config.rColor.forEach((v, i) => config.color[i] = v / 255.0);
    renderer.splat(
        radius * config.rRadius,
        config.position,
        config.velocity,
        config.color,
    );
}

const spinConfig = (radius: number, timestamp: number) => {
    // Swap X and Y because of initial offset
    config.position[0] = (1 + config.lYOffset) * canvas.width * 0.5;
    config.position[1] = (1 + config.lXOffset) * canvas.height * 0.5;
    config.velocity[0] = Math.cos(config.rRadius * timestamp + config.rStrength) * 10.0 * config.lStrength;
    config.velocity[1] = Math.sin(config.rRadius * timestamp + config.rStrength) * 10.0 * config.lStrength;
    config.lColor.forEach((v, i) => config.color[i] = v / 255.0);
    renderer.splat(
        radius * config.lRadius,
        config.position,
        config.velocity,
        config.color,
    );
}

const update = (timestamp: number) => {
    requestAnimationFrame(update);

    const radius = Math.min(canvas.width, canvas.height) * 10.0;

    if (pointer.isMoved) {
        renderer.splat(
            radius * params.pointerRadius,
            pointer.getPosition,
            pointer.getVelocity,
            pointerColor,
        );
    }

    if (!params.isPaused) {
        if (params.config == Configuration.SPELLS) spellConfig(radius);
        else if (params.config == Configuration.SPIN) spinConfig(radius, timestamp / 1000);
    }

    renderer.update(
        params.isPaused,
        timestamp / 1000,
        params.mode,
        params.iterations,
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
