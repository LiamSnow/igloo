import { createResource, Show, For } from "solid-js";
import Group from "./components/Group";
import { fetchUIData } from "./api";

function App() {
    const [data] = createResource(fetchUIData);

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
