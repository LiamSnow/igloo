use std::{sync::Arc, time::Duration};

use tokio::sync::oneshot;

use crate::{cli::model::Cli, config::BasicScriptLine, stack::IglooStack};

pub fn spawn(
    script_name: String,
    id: u32,
    istack: Arc<IglooStack>,
    uid: usize,
    args: Vec<String>,
    mut cancel_rx: oneshot::Receiver<()>,
    body: Arc<Vec<BasicScriptLine>>,
) {
    tokio::spawn(async move {
        let mut stack: &Vec<BasicScriptLine> = &body;
        let mut stack_index = 0;
        let mut is_forever = false;

        while stack_index < stack.len() {
            if cancel_rx.try_recv().is_ok() {
                break;
            }

            match stack.get(stack_index).unwrap() {
                BasicScriptLine::Command(cmd) => {
                    parse_execute(&istack, &script_name, uid, &args, cmd).await
                }
                // BasicScriptLine::HttpGet(req) => http_get(req).await,
                BasicScriptLine::Delay(ms) => {
                    tokio::time::sleep(Duration::from_millis(*ms)).await
                }
                BasicScriptLine::HttpGet { url } => http_get(&script_name, url).await,
                BasicScriptLine::HttpPost { url, body } => {
                    http_post(&script_name, url, body, &args).await
                },
                BasicScriptLine::Forever(new_body) => {
                    stack = new_body;
                    is_forever = true;
                }
            }

            stack_index += 1;
            if is_forever && stack_index == stack.len() {
                stack_index = 0;
            }
        }

        // clean up
        let mut script_states = istack.script_states.lock().await;
        script_states.current.remove(&id);
    });
}

async fn parse_execute(
    stack: &Arc<IglooStack>,
    script_name: &str,
    uid: usize,
    args: &Vec<String>,
    cmd_str: &str,
) {
    let cmd_str = match inject_args(cmd_str, args) {
        Ok(s) => s,
        Err(e) => {
            println!("Basic script {script_name} inject args into cmd error {e}");
            return;
        }
    };

    let cmd = match Cli::parse(&cmd_str) {
        Ok(r) => r,
        Err(e) => {
            println!(
                "Basic script {script_name} cmd parsing error: {}",
                e.render().to_string()
            );
            return;
        }
    };

    if let Err(err) = cmd.dispatch(&stack, uid, false).await {
        println!("Basic script {script_name} cmd failed: {err}");
    }
}

async fn http_get(script_name: &str, url: &str) {
    if let Err(err) = reqwest::get(url).await {
        println!("Basic script {} http get error: {}", script_name, err);
    }
}

async fn http_post(script_name: &str, url: &str, body: &str, args: &Vec<String>) {
    let body = match inject_args(body, args) {
        Ok(s) => s,
        Err(e) => {
            println!("Basic script {script_name} inject args into post body error {e}");
            return;
        }
    };
    let client = reqwest::Client::new();
    let res = client.post(url).body(body).send().await;
    if let Err(err) = res {
        println!("Basic script {} http post error: {}", script_name, err);
    }
}

fn inject_args(s: &str, args: &Vec<String>) -> Result<String, String> {
    let mut result = s.to_string();

    for i in 1..=9 {
        let pos_arg = format!("${}", i);
        if result.contains(&pos_arg) {
            match args.get(i - 1) {
                Some(arg) => {
                    result = result.replace(&pos_arg, arg);
                },
                None => {
                    return Err(format!("Missing pos arg ${}", i));
                }
            }
        }
    }

    Ok(result)
}
