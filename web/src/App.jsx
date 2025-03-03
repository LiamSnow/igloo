import { createSignal, onMount, onCleanup, Show, For } from "solid-js";
import { createStore } from "solid-js/store"
import Group from "./components/Group";

function App() {
    const [data, setData] = createStore(null);
    const [error, setError] = createSignal(null);
    const [execute, setExecute] = createSignal(null);

    onMount(() => {
        const ws = new WebSocket('ws://localhost:3000/ws');

        ws.onopen = () => {
            ws.send('ui get');
        };

        ws.onmessage = (event) => {
            try {
                let res = JSON.parse(event.data);
                console.log(res);

                if (res.elements !== undefined) {
                    setData(res);
                }
                else if (res.esid !== undefined) {
                    setData('states', res.esid, res.value);
                }
                else if (res.evid !== undefined) {
                    setData('values', res.evid, res.value);
                }
            } catch (err) {
                console.error('Error processing message:', err);
                setError(err);
            }
        };

        ws.onerror = (error) => {
            console.error('WebSocket error:', error);
            setError(error);
        };

        onCleanup(() => {
            ws.close();
        });

        setExecute(() => (message) => {
            if (ws.readyState === WebSocket.OPEN) {
                ws.send(message);
            } else {
                console.error('WebSocket is closed');
            }
        });
    });

    return (
        <div>
            <Show when={!data}>
                <p>Loading...</p>
            </Show>

            <Show when={error()}>
                <p>Error: {error().message}</p>
            </Show>

            <Show when={data}>
                <div class="groups">
                    <For each={data.elements || {}}>
                        {([groupName, groupItems]) => (
                            <Group
                                name={groupName}
                                items={groupItems}
                                data={data}
                                execute={execute()}
                            />
                        )}
                    </For>
                </div>
            </Show>
        </div>
    );
}

export default App;
