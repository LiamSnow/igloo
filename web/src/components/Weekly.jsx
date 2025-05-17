import istyles from './Input.module.scss';
import styles from './Weekly.module.scss';
import { unwrap } from "solid-js/store"
import { killSnake } from '../util';

function Weekly(props) {
    function changeTime(e) {
        props.execute(`weekly ${props.name} time ${e.target.value}`);
    }

    function changeDays(e) {
        let name = e.target.getAttribute("name");
        let days = unwrap(props.state()?.value.days);
        if (days.includes(name)) {
            days = days.filter(cur => cur !== name);
        }
        else {
            days.push(name);
        }
        console.log(`weekly ${props.name} days ${days.join(',')}`);
        props.execute(`weekly ${props.name} days ${days.join(',')}`);
    }

    function test(d) {
        if (props.state()?.value.days.includes(d)) {
            return styles.on
        }
        else return ""
    }

    return (
        <div>
            <h3>{killSnake(props.name)}</h3>
            <Show when={props.num_disc() > 0}>
                <div aria-label={disc_label()} title={disc_label()} style="color: #ffff77">
                    <Icon icon="mdi:warning" />
                </div>
            </Show>

            <input type="time"
                value={props.state()?.value.time}
                onChange={changeTime}
                class={istyles.Input}
                disabled={!props.state()}
            />

            <div class={styles.Weekdays}>
                <div class={test("Sunday")} name="Sunday" onClick={changeDays}>S</div>
                <div class={test("Monday")} name="Monday" onClick={changeDays}>M</div>
                <div class={test("Tuesday")} name="Tuesday" onClick={changeDays}>T</div>
                <div class={test("Wednesday")} name="Wednesday" onClick={changeDays}>W</div>
                <div class={test("Thursday")} name="Thursday" onClick={changeDays}>R</div>
                <div class={test("Friday")} name="Friday" onClick={changeDays}>F</div>
                <div class={test("Saturday")} name="Saturday" onClick={changeDays}>S</div>
            </div>

        </div>
    );
}

export default Weekly;
