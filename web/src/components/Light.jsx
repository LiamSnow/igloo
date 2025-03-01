import { createSignal, createMemo } from "solid-js";
import styles from './Light.module.scss';
import { command, hue8ToRGB, killSnake, rgbToHue8 } from '../util';

function Light(props) {
    const [light, _] = createSignal(props.state?.value?.Light || {});
    const [localHue, setLocalHue] = createSignal(null);

    const hue = createMemo(() => {
        if (localHue() !== null) {
            return localHue();
        }
        const color = light().color;
        if (color && color.r) {
            return rgbToHue8(color);
        }
        return 0;
    });

    const handleToggle = (_) => command(`light ${props.name} ${light().on ? 'off' : 'on'}`);
    const handleBrightness = (e) => command(`light ${props.name} brightness ${e.target.value}`);
    const handleTemperature = (e) => command(`light ${props.name} temp ${e.target.value}`);
    const handleColorChange = (_) => command(`light ${props.name} color ${color.r} ${color.g} ${color.b}`);
    const handleHueChange = (e) => {
        const newHue = parseInt(e.target.value);
        setLocalHue(newHue);
        handleColorChange(hue8ToRGB(newHue));
    };

    return (
        <>
            <h3>{killSnake(props.name)}</h3>
            <div style="margin: 10px 0;">
                <button
                    onClick={handleToggle}
                    style={`
                        background-color: ${light().on ? '#4CAF50' : '#f44336'};
                        color: white;
                        border: none;
                        padding: 8px 16px;
                        border-radius: 4px;
                        cursor: pointer;
                    `}
                >
                    {light().on ? 'ON' : 'OFF'}
                </button>
            </div>

            <div style="margin: 15px 0;">
                <input class={styles.BrightSlider}
                    type="range"
                    min="0"
                    max="100"
                    value={light().brightness}
                    onChange={handleBrightness}
                />
            </div>

            <Show when={!light().color_on}>
                <div style="margin: 15px 0;">
                    <input class={styles.TempSlider}
                        type="range"
                        min="0"
                        max="500"
                        value={light().temp}
                        onChange={handleTemperature}
                    />
                </div>
            </Show>

            <Show when={light().color_on}>
                <div style="margin: 15px 0;">
                    <div>
                        <input class={styles.HueSlider}
                            type="range"
                            min="0"
                            max="255"
                            value={hue()}
                            onChange={handleHueChange}
                        />
                    </div>
                </div>
            </Show>
        </>
    );
}

export default Light;
