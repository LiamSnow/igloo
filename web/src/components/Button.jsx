import styles from './Input.module.scss';
import { killSnake } from '../util';

function Button(props) {
    const cmd = props.onclick;

    function handleClick() {
        props.execute(cmd);
    }

    return (
        <div>
            <button onClick={handleClick} class={styles.Button}>{
                killSnake(props.name)
            }</button>
        </div>
    );
}

export default Button;
