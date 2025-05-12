import styles from './Input.module.scss';
import { killSnake } from '../util';

function DateTime(props) {
    function change(e) {
        props.execute(`datetime ${props.name} "${e.target.value}"`);
    }

    return (
        <div>
            <h3>{killSnake(props.name)}</h3>
            <input type="datetime-local"
                value={props.state()?.value}
                onChange={change}
                class={styles.Input}
            />
        </div>
    );
}

export default DateTime;
