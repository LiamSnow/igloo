import { createSignal, createEffect } from "solid-js";
import styles from './Light.module.scss';
import { hue8ToRGB, killSnake, rgbToHue8 } from '../util';
import { Icon } from '@iconify-icon/solid';

// https://icon-sets.iconify.design

function Light(props) {
    const [controllingBrightness, setControllingBrightness] = createSignal(true);
    const [hue, setHue] = createSignal(null);
    createEffect(() => {
        setHue(rgbToHue8(props.state().color));
    });

    const lightCmd = (s) => props.execute(`light ${props.name} ${s}`)
    const handleToggle = (_) => lightCmd(props.state().on ? 'off' : 'on');
    const handleBrightness = (e) => lightCmd(`brightness ${e.target.value}`);
    const handleTemperature = (e) => lightCmd(`temp ${e.target.value}`);
    const handleHueChange = (e) => {
        const newHue = parseInt(e.target.value);
        setHue(newHue);
        const color = hue8ToRGB(newHue);
        lightCmd(`color ${color.r} ${color.g} ${color.b}`)
    };

    const controlBrightness = (_) => setControllingBrightness(true);
    const controlColor = (_) => setControllingBrightness(false);
    const controlTemp = (_) => setControllingBrightness(false);

    return (
        <div>
            <div class={styles.Header}>
                <h3>{killSnake(props.name)}</h3>

                <button onClick={handleToggle}
                    class={ props.state().on ? styles.State + " " + styles.On : styles.State }
                >
                    <Icon icon="akar-icons:light-bulb" />
                </button>

                <button onClick={controlBrightness}>
                    <Icon icon="material-symbols-light:brightness-4" />
                </button>
                <button onClick={controlColor}>
                    <Icon icon="mdi:temperature" />
                </button>
                <button onClick={controlTemp}>
                    <Icon icon="mdi:color" />
                </button>
            </div>

            <Show when={controllingBrightness()}>
                <div>
                    <input class={styles.BrightSlider}
                        type="range"
                        min="0"
                        max="100"
                        value={props.state().brightness}
                        onChange={handleBrightness}
                    />
                </div>
            </Show>

            <Show when={!controllingBrightness() && !props.state().color_on}>
                <div>
                    <input class={styles.TempSlider}
                        type="range"
                        min="0"
                        max="500"
                        value={props.state().temp}
                        onChange={handleTemperature}
                    />
                </div>
            </Show>

            <Show when={!controllingBrightness() && props.state().color_on}>
                <div>
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
        </div>
    );
}

export default Light;
