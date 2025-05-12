import styles from './Input.module.scss';
import { killSnake } from '../util';

function Float(props) {
    function change(e) {
        props.execute(`float ${props.name} ${e.target.value}`);
    }

    return (
        <div>
            <h3>{killSnake(props.name)}</h3>
            <input type="number"
                value={props.state()?.value}
                onChange={change}
                step="0.01"
                class={styles.Input}
            />
        </div>
    );
}

export default Float;
