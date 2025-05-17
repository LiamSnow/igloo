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
                    disabled={!props.state()}
                />
                <span></span>
            </label>
            <Show when={props.num_disc() > 0}>
                <div aria-label={disc_label()} title={disc_label()} style="color: #ffff77">
                    <Icon icon="mdi:warning" />
                </div>
            </Show>
        </div>
    );
}

export default Bool;
