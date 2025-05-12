import { createSignal, onMount, onCleanup, Show, For } from "solid-js";
import { createStore } from "solid-js/store"
import Group from "./components/Group";

function Home() {
    const [data, setData] = createStore(null);
    const [error, setError] = createSignal(null);
    const [execute, setExecute] = createSignal(null);

    onMount(() => {
        const ws = new WebSocket('ws://localhost:3000/ws');

        ws.onopen = () => {
            ws.send('ui');
        };

        ws.onmessage = (event) => {
            try {
                let res = JSON.parse(event.data);
                console.log("got", res);

                // init
                if (res.elements !== undefined) {
                    setData(res);
                    setData('scripts', {});
                }

                // partial updates

                else if (res.header === "states") {
                    for (const update of res.body) {
                        setData('states', update.esid, update.value);
                    }
                }

                else if (res.header === "scripts") {
                    if ("Add" in res.body) {
                        setData('scripts', res.body.Add, true);
                    }
                    else {
                        setData('scripts', res.body.Remove, false);
                    }
                    console.log(res.body);
                    console.log("now scripts", data.scripts);
                }
            } catch (err) {
                console.error('Error processing message:', err);
                setError(err);
            }
        };

        ws.onclose = (event) => {
            if (event.code == 1008) {
                window.location.pathname = "/login";
            }
            console.log("Websocket closed", event);
        }

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

export default Home;
