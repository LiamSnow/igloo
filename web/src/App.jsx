import { createResource, onMount, onCleanup, Show, For } from "solid-js";
import Group from "./components/Group";

async function fetchUIData() {
    const response = await fetch('http://localhost:3000', {
        method: 'POST',
        headers: { 'Content-Type': 'text/plain' },
        body: 'ui get'
    });

    if (!response.ok) {
        console.error(`API error: ${response.status}`);
        return null;
    }

    return await response.json();
}

function App() {
    const [data, { mutate }] = createResource(fetchUIData);

    onMount(() => {
        const ws = new WebSocket('ws://localhost:3000/ws');

        ws.onmessage = (event) => {
            const [index, state] = JSON.parse(event.data);
            if (data()) {
                const newStates = { ...data().states };
                newStates[index] = state;
                mutate({ ...data(), states: newStates });
            }
        };

        ws.onerror = (error) => {
            console.error('WebSocket error:', error);
        };

        onCleanup(() => {
            ws.close();
        });
    });

    return (
        <div>
            <Show when={data.loading}>
                <p>Loading...</p>
            </Show>

            <Show when={data.error}>
                <p>Error: {data.error.message}</p>
            </Show>

            <Show when={data()}>
                <div class="groups">
                    <For each={Object.entries(data().elements)}>
                        {([groupName, groupItems]) => (
                            <Group
                                name={groupName}
                                items={groupItems}
                                states={data().states}
                                values={data().values}
                            />
                        )}
                    </For>
                </div>
            </Show>
        </div>
    );
}

export default App;
