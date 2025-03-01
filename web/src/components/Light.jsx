import { createSignal } from "solid-js";
import styles from './Light.module.scss';
import { killSnake } from '../util';
import { Icon } from '@iconify-icon/solid';

// https://icon-sets.iconify.design

function Light(props) {
    const [controllingBrightness, setControllingBrightness] = createSignal(true);

    const lightCmd = (s) => props.execute(`light ${props.name} ${s}`)
    const handleToggle = (_) => lightCmd(props.state().on ? 'off' : 'on');
    const handleBrightness = (e) => lightCmd(`brightness ${e.target.value}`);
    const handleTemperature = (e) => lightCmd(`temp ${e.target.value}`);
    const handleHueChange = (e) => lightCmd(`color ${parseInt(e.target.value)}`);

    const controlBrightness = (_) => setControllingBrightness(true);
    const controlColor = (_) => {
        setControllingBrightness(false);
        lightCmd(`color`);
    }
    const controlTemp = (_) => {
        setControllingBrightness(false);
        lightCmd(`temp`);
    }

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
                <button onClick={controlTemp}>
                    <Icon icon="mdi:temperature" />
                </button>
                <button onClick={controlColor}>
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
                            max="360"
                            value={props.state().hue}
                            onChange={handleHueChange}
                        />
                    </div>
                </div>
            </Show>
        </div>
    );
}

export default Light;
