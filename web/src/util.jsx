/** Fast 8-bit Hue to RGB
    Basically HSL to RGB with S=100%, L=50% */
export function hue8ToRGB(hue) {
    return {
        r: hue <= 42 ? 255 : hue <= 84 ? (85 - hue) * 6 : hue <= 169 ? 0 : hue <= 212 ? (hue - 170) * 6 : 255,
        g: hue <= 42 ? hue * 6 : hue <= 127 ? 255 : hue <= 169 ? (170 - hue) * 6 : 0,
        b: hue <= 84 ? 0 : hue <= 127 ? (hue - 85) * 6 : hue <= 212 ? 255 : (255 - hue) * 6,
    }
}

/** Takes color { r: u8, g: u8, b: u8 }
    Returns a hue 0-255 */
export function rgbToHue8(color) {
    if (!color || !color.r) {
        return 0;
    }

    const { r, g, b } = color;
    const max = Math.max(r, g, b);
    const min = Math.min(r, g, b);
    const chroma = max - min;
    if (chroma === 0) return 0;
    let hue;
    if (max === r) {
        hue = ((g - b) / chroma + 6) % 6;
    }
    else if (max === g) {
        hue = (b - r) / chroma + 2;
    }
    else {
        hue = (r - g) / chroma + 4;
    }
    return Math.round(hue * 42.5) % 256;
}

/** Converts a snake_case_name to a fancy Snake Case Name */
export function killSnake(s) {
    if (!s) return '';
    return s.split('_')
        .map(word => {
            if (word.length === 0) return '';
            return word.charAt(0).toUpperCase() + word.slice(1);
        })
        .join(' ');
}
