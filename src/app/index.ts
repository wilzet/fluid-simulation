import * as dat from "dat.gui";
import { Renderer, Resolution, Mode } from "fluid-simulation";
import { resizeCanvas, isMobile, pixelScaling, randomColor, defaultBlueColor, defaultRedColor } from "./utils";
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
    color: defaultBlueColor.slice(),
    useRandomColor: true,
    config: Configuration.NONE,
    obstacle: false,
};
let wasPaused = false;

const config = {
    position: new Float32Array([0.0, 0.0]),
    velocity: new Float32Array([0.0, 0.0]),
    color: new Float32Array([0.0, 0.0, 0.0]),
    lColor: defaultRedColor.slice(),
    lRadius: 0.2,
    lStrength: 10.0,
    lXOffset: 0.0,
    lYOffset: 0.0,
    rColor: defaultBlueColor.slice(),
    rRadius: 0.2,
    rStrength: 10.0,
    rXOffset: 0.0,
    rYOffset: 0.0,
    obstacleColor: [0, 0, 0],
    obstacleRadius: 100.0,
    obstacleXOffset: 0.0,
    obstacleYOffset: 0.0,
    obstacleCircle: true,
};
const pointer = new Pointer([0, 0]);
const pointerColor = new Float32Array(params.color.map((v) => v / 255.0));

const canvasId = "canvas";
const canvas = document.getElementById(canvasId) as HTMLCanvasElement;
resizeCanvas(canvas);

const renderer = Renderer.create(canvasId, params.simResolution, params.dyeResolution);

const resizeSimulation = () => {
    resizeCanvas(canvas);
    renderer.resize(params.simResolution, params.dyeResolution);
}

const generateColor = () => {
    if (!params.useRandomColor) return;

    randomColor(undefined, undefined, 0.5, 0.9, 0.3, 0.5).forEach((v, i) => {
        pointerColor[i] = v;
        params.color[i] = v * 255.0;
    });
}

const createGUI = () => {
    const resolutions = {
        "Ultra+": Resolution.ONE,
        "Ultra": Resolution.TWO,
        "High": Resolution.FOUR,
        "Medium": Resolution.EIGHT,
        "Low": Resolution.SIXTEEN,
    };
    const gui = new dat.GUI({ closeOnTop: true, hideable: true });

    const visualsFolder = gui.addFolder("Visuals");
    visualsFolder.add(
        params,
        "mode",
        {
            "Dye": Mode.DYE,
            "Velocity": Mode.VELOCITY,
        },
    )
        .name("Mode");
    visualsFolder.add(params, "dyeResolution", resolutions)
        .name("Quality")
        .onFinishChange(resizeSimulation);
    visualsFolder.open();

    const simulationFolder = gui.addFolder("Simulation");
    simulationFolder.add(params, "viscosity", 0.0, 5.0, 0.01).name("Viscosity");
    simulationFolder.add(params, "dissipation", 0.0, 5.0, 0.01).name("Dye diffusion");
    simulationFolder.add(params, "curl", 0.0, 2.0, 0.01).name("Vorticity amount");
    simulationFolder.open();

    const advancedFolder = simulationFolder.addFolder("Advanced");
    advancedFolder.add(params, "simResolution", resolutions)
        .name("Simulation quality")
        .onFinishChange((value: number) => {
            params.iterations = isMobile() ? 20 : value <= 3 ? value <= 2 ? value <= 1 ? 50 : 40 : 30 : 20;
            resizeSimulation();
        });
    advancedFolder.add(params, "pressure", 0.0, 1.0, 0.01).name("Pressure");
    advancedFolder.add(params, "iterations", 10, isMobile() ? 50 : 80, 1).name("Solver iterations").listen();
    
    const configurationFolder = gui.addFolder("Configuration");
    let settingsFolder: dat.GUI | undefined;
    let obstacleFolder: dat.GUI | undefined;
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
                config.lRadius = config.rRadius = 0.2;
                config.lStrength = config.rStrength = 10.0;
                config.lXOffset = -0.9;
                config.rXOffset = 0.9;
                config.lYOffset = config.rYOffset = 0.0;

                settingsFolder = configurationFolder.addFolder("Spell Settings");
                
                const leftFolder = settingsFolder.addFolder("Left");
                leftFolder.addColor(config, "lColor").name("Color");
                leftFolder.add(config, "lRadius", 0.01, 2.0, 0.01).name("Radius");
                leftFolder.add(config, "lStrength", 0.0, 100.0, 0.01).name("Strength");
                leftFolder.add(config, "lXOffset", -1.0, 1.0, 0.01).name("X");
                leftFolder.add(config, "lYOffset", -1.0, 1.0, 0.01).name("Y");

                const rightFolder = settingsFolder.addFolder("Right");
                rightFolder.addColor(config, "rColor").name("Color");
                rightFolder.add(config, "rRadius", 0.01, 2.0, 0.01).name("Radius");
                rightFolder.add(config, "rStrength", 0.0, 100.0, 0.01).name("Strength");
                rightFolder.add(config, "rXOffset", -1.0, 1.0, 0.01).name("X");
                rightFolder.add(config, "rYOffset", -1.0, 1.0, 0.01).name("Y");

                settingsFolder.open();
            } else if (value == Configuration.SPIN) {
                config.lRadius = 0.4;
                config.lStrength = 10.0;
                config.lXOffset = config.lYOffset = 0.0;
                config.rRadius = 0.5 * Math.PI;
                config.rStrength = 0.0;

                settingsFolder = configurationFolder.addFolder("Spin Settings");
                settingsFolder.add(config, "rRadius", -2 * Math.PI, 2 * Math.PI, 0.01).name("Rotation speed");
                settingsFolder.add(config, "rStrength", -Math.PI, Math.PI, 0.01).name("Angular offset");
                settingsFolder.addColor(config, "lColor").name("Color");
                settingsFolder.add(config, "lRadius", 0.01, 3.0, 0.01).name("Radius");
                settingsFolder.add(config, "lStrength", 0.0, 100.0, 0.01).name("Strength");
                settingsFolder.add(config, "lXOffset", -1.0, 1.0, 0.01).name("X");
                settingsFolder.add(config, "lYOffset", -1.0, 1.0, 0.01).name("Y");

                settingsFolder.open();
            }
        });
    configurationFolder.add(params, "obstacle").name("Use obstacle").onFinishChange((value: boolean) => {
        if (obstacleFolder) {
            configurationFolder.removeFolder(obstacleFolder);
            obstacleFolder = undefined;
        }

        if (!value) {
            renderer.set_obstacle(
                undefined,
                config.position,
                config.color,
                true,
            );
        } else {
            config.obstacleRadius = 100.0;
            config.obstacleXOffset = 0.0;
            config.obstacleYOffset = 0.0;
            
            obstacleFolder = configurationFolder.addFolder("Obstacle Settings");
            obstacleFolder.addColor(config, "obstacleColor").name("Color");
            obstacleFolder.add(config, "obstacleRadius", 0.01, Math.max(canvas.width, canvas.height) * 0.5, 1.0).name("Radius");
            obstacleFolder.add(config, "obstacleXOffset", -1.0, 1.0, 0.01).name("X");
            obstacleFolder.add(config, "obstacleYOffset", -1.0, 1.0, 0.01).name("Y");
            obstacleFolder.add(config, "obstacleCircle").name("Use circle");
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

    gui.add(params, "isPaused").name("Pause").onFinishChange(() => wasPaused = !wasPaused).listen();

    const gitHub = gui.add({ fun: () => window.open("https://github.com/wilzet/fluid-simulation") }, "fun").name("GitHub");
    const icon = document.createElement("div");
    icon.className = "github";
    const list = gitHub.domElement.parentElement?.parentElement;
    if (list) list.className += " link";
    list?.appendChild(icon);

    if (isMobile()) gui.close();
}

const spellConfig = (radius: number) => {
    const halfWidth = canvas.width * 0.5;
    const halfHeight = canvas.height * 0.5;
    config.velocity[1] = 0.0;

    // LEFT
    config.position[0] = (1.0 + config.lXOffset) * halfWidth;
    config.position[1] = (1.0 + config.lYOffset) * halfHeight;
    config.velocity[0] = 10.0 * config.lStrength;
    config.lColor.forEach((v, i) => config.color[i] = v / 255.0);
    renderer.splat(
        radius * config.lRadius,
        config.position,
        config.velocity,
        config.color,
    );

    // RIGHT
    config.position[0] = (1.0 + config.rXOffset) * halfWidth;
    config.position[1] = (1.0 + config.rYOffset) * halfHeight;
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
    config.position[0] = (1 + config.lXOffset) * canvas.width * 0.5;
    config.position[1] = (1 + config.lYOffset) * canvas.height * 0.5;
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

const obstacleConfig = () => {
    config.position[0] = (1 + config.obstacleXOffset) * canvas.width * 0.5;
    config.position[1] = (1 + config.obstacleYOffset) * canvas.height * 0.5;
    config.obstacleColor.forEach((v, i) => config.color[i] = v / 255.0);
    renderer.set_obstacle(
        config.obstacleRadius,
        config.position,
        config.color,
        config.obstacleCircle,
    );
}

const update = (timestamp: number) => {
    requestAnimationFrame(update);

    const radius = Math.min(canvas.width, canvas.height) * 10.0;

    if (params.obstacle) obstacleConfig();

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
    window.addEventListener("resize", resizeSimulation);

    document.addEventListener("visibilitychange", () => {
        if (document.hidden) {
            params.isPaused = true;
            pointer.up();
        } else {
            params.isPaused = wasPaused;
        }
    });

    window.addEventListener("keydown", (e) => {
        if (e.code === "Space") params.isPaused = !params.isPaused;
    });

    pointerEvents();

    createGUI();

    requestAnimationFrame(update);
}

run();
