import styles from './Button.module.scss';
import { command, killSnake } from '../util';

function Button(props) {
    const cmd = props.onclick;

    function handleClick() {
        command(cmd);
    }

    return (
        <>
            <h3>{killSnake(props.name)}</h3>
            <button on:click={handleClick}>Trigger</button>
        </>
    );
}

export default Button;
