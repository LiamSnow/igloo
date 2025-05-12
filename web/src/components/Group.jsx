import { For } from "solid-js";
import Light from "./Light";
import Int from "./Int";
import Float from "./Float";
import Time from "./Time";
import DateTime from "./DateTime";
import Weekly from "./Weekly";
import Text from "./Text";
import Bool from "./Bool";
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

                        //TODO: add support for other Light types
                        if (configType === "RGBCTLight") {
                            const state = () => props.data.states[item.esid]?.value?.Light;
                            return <Light name={item.cfg[configType]}
                                execute={props.execute}
                                state={state}
                            />;
                        }

                        if (configType === "Int") {
                            const state = () => props.data.states[item.esid]?.value?.Int;
                            return <Int name={item.cfg[configType]}
                                execute={props.execute}
                                state={state}
                            />;
                        }

                        if (configType === "Float") {
                            const state = () => props.data.states[item.esid]?.value?.Float;
                            return <Float name={item.cfg[configType]}
                                execute={props.execute}
                                state={state}
                            />;
                        }

                        if (configType === "Time") {
                            const state = () => props.data.states[item.esid]?.value?.Time;
                            return <Time name={item.cfg[configType]}
                                execute={props.execute}
                                state={state}
                            />;
                        }

                        if (configType === "DateTime") {
                            const state = () => props.data.states[item.esid]?.value?.DateTime;
                            return <DateTime name={item.cfg[configType]}
                                execute={props.execute}
                                state={state}
                            />;
                        }

                        if (configType === "Weekly") {
                            const state = () => props.data.states[item.esid]?.value?.Weekly;
                            return <Weekly name={item.cfg[configType]}
                                execute={props.execute}
                                state={state}
                            />;
                        }

                        if (configType === "Text") {
                            const state = () => props.data.states[item.esid]?.value?.Text;
                            return <Text name={item.cfg[configType]}
                                execute={props.execute}
                                state={state}
                            />;
                        }

                        if (configType === "Bool") {
                            const state = () => props.data.states[item.esid]?.value?.Bool;
                            return <Bool name={item.cfg[configType]}
                                execute={props.execute}
                                state={state}
                            />;
                        }

                        //TODO climate, fan

                        return <p>Unknown component type: {configType}</p>;
                    }}
                </For>
            </div>
        </div>
    );
}

export default Group;
