import { createSignal,  } from 'solid-js';
import istyles from './Input.module.scss';
import styles from './Terminal.module.scss';

function Terminal(props) {
    const [inputValue, setInputValue] = createSignal('');

    const handleSubmit = (e) => {
        e.preventDefault();
        props.execute(inputValue());
        setInputValue('');
    };

    const convres = () => {
        if (typeof props.result === "object") {
            return JSON.stringify(props.result);
        }
        return props.result;
    };

    return (
        <div>
            <h3>Terminal</h3>

            <div class={styles.Response}>
                { convres() }
            </div>

            <form class={styles.Form} onSubmit={handleSubmit}>
                <input
                    class={istyles.Input}
                    value={inputValue()}
                    onInput={(e) => setInputValue(e.target.value)}
                />
                <button class={istyles.Button} type="submit">Run</button>
            </form>
        </div>
    );
}

export default Terminal;
