use leptos::prelude::*;
use leptos_bevy_canvas::prelude::*;

use crate::editor::TextEvent;
use crate::editor::init_bevy_app;

#[component]
pub fn App() -> impl IntoView {
    let (text_event_sender, bevy_text_receiver) = event_l2b::<TextEvent>();

    let on_input = move |evt| {
        text_event_sender
            .send(TextEvent {
                text: event_target_value(&evt),
            })
            .ok();
    };

    view! {
        <input type="text" on:input=on_input />
        <br />

        <BevyCanvas
            init=move || {
                init_bevy_app(bevy_text_receiver)
            }

            {..}
            width="300"
            height="500"
        />
    }
}
