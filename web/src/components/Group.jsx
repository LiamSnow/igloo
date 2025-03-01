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
                        const name = typeof item.cfg[configType] === 'string'
                            ? item.cfg[configType]
                            : item.cfg[configType].name;

                        const state = item.esid !== null ? props.states[item.esid] : null;
                        const value = item.evid !== null ? props.values[item.evid] : null;

                        let component;
                        switch (configType) {
                            case "RGBCTLight":
                                component = <Light name={name} state={state} />;
                                break;
                            case "TimeSelector":
                                component = <TimeSelector name={name} value={value} />;
                                break;
                            case "Button":
                                component = <Button name={name} onclick={item.cfg.on_click} />;
                                break;
                            default:
                                component = <p>Unknown component type: {configType}</p>;
                        }
                        return (
                            <div>
                                {component}
                            </div>
                        );
                    }}
                </For>
            </div>
        </div>
    );
}

export default Group;
