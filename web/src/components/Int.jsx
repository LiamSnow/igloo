import styles from './Input.module.scss';
import { killSnake } from '../util';

function Int(props) {
    function change(e) {
        props.execute(`int ${props.name} ${e.target.value}`);
    }

    return (
        <div>
            <h3>{killSnake(props.name)}</h3>
            <input type="number"
                value={props.state()?.value}
                onChange={change}
                step="1"
                class={styles.Input}
            />
        </div>
    );
}

export default Int;
