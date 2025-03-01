import { createSignal } from "solid-js";
import styles from './TimeSelector.module.scss';
import { command, killSnake } from '../util';

function TimeSelector(props) {
    const timeValue = props.value?.Time || "";
    const [time, setTime] = createSignal(timeValue);

    function handleTimeChange(e) {
        const newTime = e.target.value;
        setTime(newTime);
        command(`time ${props.name} ${newTime}`);
    }

    return (
        <>
            <h3>{killSnake(props.name)}</h3>
            <input type="time" value={time()} onChange={handleTimeChange} />
        </>
    );
}

export default TimeSelector;
