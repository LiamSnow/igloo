import { For } from "solid-js";
import Light from "./Light";
import TimeSelector from "./TimeSelector";
import Button from "./Button";
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
                        console.log(configType);

                        switch (configType) {
                            case "RGBCTLight":
                                const state = () => props.data.states[item.esid]?.value?.Light;
                                return <Light name={item.cfg[configType]}
                                                execute={props.execute}
                                                state={state}
                                            />;
                            case "TimeSelector":
                                const value = () => props.data.values[item.evid].Time;
                                return <TimeSelector name={item.cfg[configType].name}
                                                execute={props.execute}
                                                group={props.name}
                                                value={value}
                                            />;
                            case "Button":
                                return <Button name={item.cfg[configType].name}
                                                execute={props.execute}
                                                onclick={item.cfg[configType].on_click}
                                            />;
                            default:
                                return <p>Unknown component type: {configType}</p>;
                        }
                    }}
                </For>
            </div>
        </div>
    );
}

export default Group;
