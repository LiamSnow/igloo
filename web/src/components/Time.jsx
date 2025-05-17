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
                disabled={!props.state()}
            />
            <Show when={props.num_disc() > 0}>
                <div aria-label={disc_label()} title={disc_label()} style="color: #ffff77">
                    <Icon icon="mdi:warning" />
                </div>
            </Show>
        </div>
    );
}

export default Time;
