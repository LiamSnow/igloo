import { For } from "solid-js";
import Light from "./Light";
import TimeSelector from "./TimeSelector";
import Button from "./Button";
import Script from "./Script";
import styles from './Group.module.scss';
import { killSnake } from '../util';

function Group(props) {
    return (
        <div class={styles.Container}>
            <h2>{killSnake(props.name)}</h2>
            <div>
                <For each={props.items}>
                    {(item) => {
                        const configType = Object.keys(item.cfg)[0];

                        if (configType === "Button") {
                            return <Button name={item.cfg[configType][0]}
                                execute={props.execute}
                                onclick={item.cfg[configType][1]}
                            />;
                        }

                        if (configType === "Script") {
                            const running = () => props.data.scripts &&
                                                  item.sid in props.data.scripts &&
                                                  props.data.scripts[item.sid];
                            return <Script name={item.cfg[configType]}
                                execute={props.execute}
                                running={running}
                                sid={item.sid}
                            />;
                        }


                        if (configType === "RGBCTLight") {
                            const state = () => props.data.states[item.esid]?.value?.Light;
                            return <Light name={item.cfg[configType]}
                                execute={props.execute}
                                state={state}
                            />;
                        }

                        if (configType === "TimeSelector") {
                            const state = () => props.data.states[item.esid]?.value?.Time;
                            return <TimeSelector name={item.cfg[configType].name}
                                execute={props.execute}
                                group={props.name}
                                state={state}
                            />;
                        }

                        return <p>Unknown component type: {configType}</p>;
                    }}
                </For>
            </div>
        </div>
    );
}

export default Group;
