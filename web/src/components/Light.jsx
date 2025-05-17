import { createSignal } from "solid-js";
import styles from './Light.module.scss';
import { killSnake } from '../util';
import { Icon } from '@iconify-icon/solid';

// https://icon-sets.iconify.design

function Light(props) {
    const [controllingBrightness, setControllingBrightness] = createSignal(true);

    const lightCmd = (s) => props.execute(`light ${props.name} ${s}`)
    const handleToggle = (_) => lightCmd(props.state()?.on ? 'off' : 'on');
    const handleBrightness = (e) => lightCmd(`brightness ${e.target.value}`);
    const handleTemperature = (e) => lightCmd(`temp ${e.target.value}`);
    const handleHueChange = (e) => lightCmd(`color ${e.target.value}`);

    const controlBrightness = (_) => setControllingBrightness(true);
    const controlColor = (_) => {
        setControllingBrightness(false);
        lightCmd(`color`);
    }
    const controlTemp = (_) => {
        setControllingBrightness(false);
        lightCmd(`temp`);
    }

    const disc_label = () => {
        return `${props.num_disc()} device(s) are disconnected`
    }

    return (
        <div>
            <div class={styles.Header}>
                <h3>{killSnake(props.name)}</h3>

                <button onClick={handleToggle} class={styles.State} disabled={!props.state()}>
                    <Show when={props.state()?.on} fallback={
                        <Icon icon="mdi:toggle-switch-variant-off" />
                    }>
                        <Icon icon="mdi:toggle-switch-variant" />
                    </Show>
                </button>

                <button onClick={controlBrightness} disabled={!props.state()}>
                    <Icon icon="material-symbols-light:brightness-4" />
                </button>

                <Show when={props.type.includes("CT")}>
                    <button onClick={controlTemp}
                            disabled={!props.state()}
                            class={props.state()?.color_on ? styles.Off : ""}>
                        <Icon icon="mdi:temperature" />
                    </button>
                </Show>

                <Show when={props.type.includes("RGB")}>
                    <button onClick={controlColor}
                            disabled={!props.state()}
                            class={props.state()?.color_on ? "" : styles.Off}>
                        <Icon icon="mdi:color" />
                    </button>
                </Show>

                <Show when={props.num_disc() > 0}>
                    <div aria-label={disc_label()} title={disc_label()} style="color: #ffff77">
                        <Icon icon="mdi:warning" />
                    </div>
                </Show>
            </div>

            <Show when={controllingBrightness()}>
                <div class={styles.SliderBox}>
                    <input class={styles.BrightSlider}
                        type="range"
                        min="0"
                        max="100"
                        value={props.state()?.brightness}
                        onChange={handleBrightness}
                        disabled={!props.state()}
                    />
                </div>
            </Show>

            <Show when={!controllingBrightness() && !props.state()?.color_on}>
                <div class={styles.SliderBox}>
                    <input class={styles.TempSlider}
                        type="range"
                        min="0"
                        max="500"
                        value={props.state()?.temp}
                        onChange={handleTemperature}
                        disabled={!props.state()}
                    />
                </div>
            </Show>

            <Show when={!controllingBrightness() && props.state()?.color_on}>
                <div class={styles.SliderBox}>
                    <input class={styles.HueSlider}
                        type="range"
                        min="0"
                        max="360"
                        value={props.state()?.hue}
                        onChange={handleHueChange}
                        disabled={!props.state()}
                    />
                </div>
            </Show>
        </div>
    );
}

export default Light;
