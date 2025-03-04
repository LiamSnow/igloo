use std::{error::Error, sync::Arc, time::Duration};

use tokio::{sync::oneshot, time};

use crate::{
    cli::model::LightAction, stack::IglooStack, selector::Selection,
};

pub async fn spawn(
    id: u32,
    stack: Arc<IglooStack>,
    uid: usize,
    args: Vec<String>,
    mut cancel_rx: oneshot::Receiver<()>,
) -> Result<(), Box<dyn Error>> {
    //parse args
    if args.len() < 2 || args.len() > 3 {
        return Err("Usage `{selection} {speed} {length_ms (optional)}`".into());
    }
    let sel = Selection::from_str(&stack.dev_lut, args.get(0).unwrap())?;
    if !stack.perms.has_perm(&sel, uid) {
        return Err("NOT AUTHORIZED".into());
    }
    let speed: u8 = args.get(1).unwrap().parse()?;
    let length_ms: Option<u32>;
    if let Some(lms) = args.get(2) {
        length_ms = Some(lms.parse()?);
    } else {
        length_ms = None;
    }

    //precalc
    //255 => 1000ms
    //0 => 10ms
    let step_ms = (((255 - speed) as f32 / 255.) * 100. + 10.) as u32;
    let mut interval = time::interval(Duration::from_millis(step_ms.into()));
    let mut hue = 0;
    let num_steps = length_ms.and_then(|l| Some(l / step_ms));
    let mut step_num = 0;

    //run
    tokio::spawn(async move {
        loop {
            interval.tick().await;
            if cancel_rx.try_recv().is_ok() {
                break;
            }

            sel
                .execute(&stack, LightAction::Color { hue: Some(hue) }.into())
                .unwrap(); //FIXME
            hue = (hue + 1) % 255;
            if let Some(num_steps) = num_steps {
                step_num += 1;
                if step_num > num_steps {
                    break;
                }
            }
        }

        // clean up
        let mut script_states = stack.script_states.lock().await;
        script_states.current.remove(&id);

        println!("rainbow stopped!!");
    });

    Ok(())
}
