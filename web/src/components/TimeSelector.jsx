import { createSignal } from "solid-js";
import styles from './TimeSelector.module.scss';
import { killSnake } from '../util';

function TimeSelector(props) {
    function handleTimeChange(e) {
        props.execute(`ui set ${props.group}.${props.name} ${e.target.value}`);
    }

    return (
        <div>
            <h3>{killSnake(props.name)}</h3>
            <input type="time" value={props.value()} onChange={handleTimeChange} />
        </div>
    );
}

export default TimeSelector;
