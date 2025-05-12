import styles from './Input.module.scss';
import { killSnake } from '../util';

function Bool(props) {
    function change(e) {
        props.execute(`bool ${props.name} "${e.target.checked}"`);
    }

    return (
        <div>
            <h3>{killSnake(props.name)}</h3>
            <label class={styles.Switch}>
                <input checked={props.state() == "True"}
                    onChange={change}
                    type="checkbox"
                />
                <span></span>
            </label>
        </div>
    );
}

export default Bool;
