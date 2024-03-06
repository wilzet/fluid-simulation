export const defaultBlueColor = [47, 161, 214];
export const defaultRedColor = [214, 61, 47];

export function isMobile() {
    const toMatch = [
        /Android/i,
        /webOS/i,
        /iPhone/i,
        /iPad/i,
        /iPod/i,
        /BlackBerry/i,
        /Windows Phone/i
    ];
    
    return toMatch.some((toMatchItem) => navigator.userAgent.match(toMatchItem));
}

export function pixelScaling(value: number) {
    return Math.floor(value * (window.devicePixelRatio || 1));
}

export function randomColor(
    minHue: number = 0.0,
    maxHue: number = 360.0,
    minSaturation: number = 0.0,
    maxSaturation: number = 1.0,
    minLightness: number = 0.0,
    maxLightness: number = 1.0,
) {
    if (maxHue < minHue) [minHue, maxHue] = [maxHue, minHue];
    if (maxSaturation < minSaturation) [minSaturation, maxSaturation] = [maxSaturation, minSaturation];
    if (maxLightness < minLightness) [minLightness, maxLightness] = [maxLightness, minLightness];

    const hue = randomRange(minHue, maxHue);
    const saturation = randomRange(minSaturation, maxSaturation);
    const lightness = randomRange(minLightness, maxLightness);

    const f = (n: number) => {
        const k = (n + hue / 30.0) % 12.0;
        const a = saturation * Math.min(lightness, 1.0 - lightness);

        return lightness - a * Math.max(-1, Math.min(k - 3, 9 - k, 1));
    }

    return [f(0), f(8), f(4)];
}

function randomRange(min: number, max: number) {
    return min + Math.random() * (max - min);
}