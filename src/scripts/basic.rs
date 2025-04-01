use std::{sync::Arc, time::Duration};

use tokio::sync::oneshot;
use tracing::{error, info, span, Level};

use crate::{cli::model::Cli, config::BasicScriptLine, state::IglooState};

const MAX_ARGS: usize = 9;

pub fn spawn(
    script_name: String,
    id: u32,
    istate: Arc<IglooState>,
    uid: Option<usize>,
    mut args: Vec<String>,
    mut cancel_rx: oneshot::Receiver<()>,
    body: Vec<BasicScriptLine>,
) {
    tokio::spawn(async move {
        let span = span!(Level::INFO, "Builtin Script", script_name, id);
        let _enter = span.enter();
        info!("running uid={:#?}, args={:#?}", uid, args);

        let mut state: &Vec<BasicScriptLine> = &body;
        let mut state_index = 0;
        let mut is_forever = false;

        while state_index < state.len() {
            if cancel_rx.try_recv().is_ok() {
                break;
            }

            match state.get(state_index).unwrap() {
                BasicScriptLine::Command(cmd) => {
                    parse_execute(&istate, uid, &args, cmd).await
                }
                BasicScriptLine::Delay(ms) => tokio::time::sleep(Duration::from_millis(*ms)).await,
                BasicScriptLine::HttpGet { url } => http_get(url).await,
                BasicScriptLine::HttpPost { url, body } => http_post(url, body, &args).await,
                BasicScriptLine::Forever(new_body) => {
                    state = new_body;
                    is_forever = true;
                }
                BasicScriptLine::Set(k, v) => {
                    if *k > MAX_ARGS {
                        panic!("Basic script {script_name}: save at index {k} is > max index {MAX_ARGS}");
                    }

                    if args.len() <= *k {
                        args.resize(*k + 1, "NULL".to_string());
                    }

                    args[*k] = v.clone();
                }
            }

            state_index += 1;
            if is_forever && state_index == state.len() {
                state_index = 0;
            }
        }

        // clean up
        let mut script_states = istate.scripts.states.lock().await;
        script_states.current.remove(&id);
    });
}

async fn parse_execute(
    state: &Arc<IglooState>,
    uid: Option<usize>,
    args: &Vec<String>,
    cmd_str: &str,
) {
    let cmd_str = match inject_args(cmd_str, args) {
        Ok(s) => s,
        Err(e) => {
            error!("argument injection: {e}");
            return;
        }
    };

    let cmd = match Cli::parse(&cmd_str) {
        Ok(r) => r,
        Err(e) => {
            error!("command parsing error: {}", e.render().to_string());
            return;
        }
    };

    if let Err(err) = cmd.dispatch(&state, uid, false).await {
        error!("command dispatch: {err}");
    }
}

async fn http_get(url: &str) {
    if let Err(err) = reqwest::get(url).await {
        error!("HTTP GET: {}", err);
    }
}

async fn http_post(url: &str, body: &str, args: &Vec<String>) {
    let body = match inject_args(body, args) {
        Ok(s) => s,
        Err(e) => {
            error!("HTTP POST inject arguments: {e}");
            return;
        }
    };
    let client = reqwest::Client::new();
    let res = client.post(url).body(body).send().await;
    if let Err(err) = res {
        error!("HTTP POST: {}", err);
    }
}

fn inject_args(s: &str, args: &Vec<String>) -> Result<String, String> {
    let mut result = s.to_string();

    for i in 1..=MAX_ARGS {
        let pos_arg = format!("${}", i);
        if result.contains(&pos_arg) {
            match args.get(i - 1) {
                Some(arg) => {
                    result = result.replace(&pos_arg, arg);
                }
                None => {
                    return Err(format!("Missing pos arg ${}", i));
                }
            }
        }
    }

    Ok(result)
}
