import { Mode, Resolution } from "fluid-simulation";

export enum Config {
    NONE,
    SPELL,
}

export interface Params {
    isPaused: boolean,
    mode: Mode,
    dyeResolution: Resolution,
    simResolution: Resolution,
    pointerRadius: number,
    pointerStrength: number,
    iterations: number,
    viscosity: number,
    dissipation: number,
    curl: number,
    pressure: number,
    color: number[],
    useRandomColor: boolean,
    config: Config,
}