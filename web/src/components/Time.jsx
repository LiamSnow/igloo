import styles from './Input.module.scss';
import { killSnake } from '../util';

function Time(props) {
    function change(e) {
        props.execute(`time ${props.name} "${e.target.value}"`);
    }

    return (
        <div>
            <h3>{killSnake(props.name)}</h3>
            <input type="time"
                value={props.state()?.value}
                onChange={change}
                class={styles.Input}
            />
        </div>
    );
}

export default Time;
