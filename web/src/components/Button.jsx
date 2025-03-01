import styles from './Button.module.scss';
import { killSnake } from '../util';

function Button(props) {
    const cmd = props.onclick;

    function handleClick() {
        props.execute(cmd);
    }

    return (
        <div>
            <h3>{killSnake(props.name)}</h3>
            <button on:click={handleClick}>Trigger</button>
        </div>
    );
}

export default Button;
