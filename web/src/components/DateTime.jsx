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

export default DateTime;
