use maud::{DOCTYPE, Markup, PreEscaped, html};

pub fn wrap_page(content: Markup) -> Markup {
    html! {
        (DOCTYPE)
        html lang = "en" {
            head {
                title { "Igloo" }
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1.0";
                style {
                    (PreEscaped(BASE_THEME))
                }
            }
        }

        noscript {
            h1 {
                "Please enable JavaScript!"
            }
        }

        header {
            div {
                h1 {
                    "Igloo"
                }
            }
            div {
                a href="/dashboards" {
                    "Dashboards"
                }
                a href="/penguin" {
                    "Penguin"
                }
                a href="/settings" {
                    "Settings"
                }
            }
        }

        main {
            (content)
        }

        script {
            (PreEscaped(JS))
        }
    }
}

pub const JS: &str = r#"
    const ws = new WebSocket('ws://localhost:3000/ws');
    
    ws.onopen = () => {
        console.log("WS Connected");
    };
    
    ws.onmessage = (event) => {
        console.log("WS Sent:", event.data);
        const parts = event.data.split(",");
        if (parts.length != 3) {
            console.error("invalid msg:",event.data);
            return;
        }
        document.getElementById(parts[1]).value = parts[2];
    };
    
    ws.onerror = (error) => {
        console.error("WS Error:", error);
    };
    
    ws.onclose = () => {
        console.error("WS Closed");
    };

    function setValue(e, dash_id, elid) {
        console.log("Sending dash_id=", dash_id, ",elid=", elid);
        ws.send(`${dash_id},${elid},${e.target.value}`);
    }
"#;

pub const BASE_THEME: &str = r#"
    *, *:before, *:after {
        box-sizing: border-box;
        font-family: monospace;
    }

    body {
        margin: 0;
        padding: 0;
        color: white;
        background: #111;
    }

    header {
        width: 100%;
        background: #444;
        height: 42px;
        display: flex;
        justify-content: space-between;
        align-items: center;
        padding: 0 6%;
    }

    header h1 {
        margin: 0;
    }

    header a {
        text-decoration: none;
        color: white;
    }
"#;
