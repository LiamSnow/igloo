import styles from './Script.module.scss';
import { killSnake, firstWord } from '../util';

function Script(props) {

    function run() {
        props.execute(`script run-with-id ${props.sid} ${props.name}`);
    }

    function cancel() {
        props.execute(`script cancel ${props.sid}`);
    }

    return (
        <div class={styles.Container}>
            {killSnake(firstWord(props.name))}

            <Show when={props.running()} fallback={
                <button onClick={run} class={styles.Button}>Run</button>
            }>
                <button onClick={cancel} class={styles.Button}>Cancel</button>
            </Show>

        </div>
    );
}

export default Script;
