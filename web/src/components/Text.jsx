import styles from './Input.module.scss';
import { killSnake } from '../util';

function Text(props) {
    function change(e) {
        props.execute(`text ${props.name} "${e.target.value}"`);
    }

    return (
        <div>
            <h3>{killSnake(props.name)}</h3>
            <input value={props.state()?.value}
                onChange={change}
                class={styles.Input}
            />
        </div>
    );
}

export default Text;
