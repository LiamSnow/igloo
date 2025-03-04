use std::{error::Error, sync::Arc, time::Duration};

use tokio::{sync::oneshot, time};

use crate::{
    cli::model::LightAction, map::IglooStack, selector::Selection,
};

pub async fn spawn(
    id: u32,
    stack: Arc<IglooStack>,
    uid: usize,
    args: Vec<String>,
    mut cancel_rx: oneshot::Receiver<()>,
) -> Result<(), Box<dyn Error>> {
    //parse args
    if args.len() != 4 {
        return Err("Wrong number of args".into());
    }
    let sel = Selection::from_str(&stack.dev_lut, args.get(0).unwrap())?;
    if !stack.perms.has_perm(&sel, uid) {
        return Err("NOT AUTHORIZED".into());
    }
    let start_brightness: u8 = args.get(1).unwrap().parse()?;
    let end_brightness: u8 = args.get(2).unwrap().parse()?;
    let length_ms: u32 = args.get(3).unwrap().parse()?;

    //precalc
    let step_ms = 50;
    let num_steps = length_ms / step_ms;
    let step_brightness = ((end_brightness - start_brightness) as f32) / num_steps as f32;
    let mut brightness = start_brightness as f32;
    let mut interval = time::interval(Duration::from_millis(step_ms.into()));

    //run
    tokio::spawn(async move {
        for _ in 0..num_steps {
            interval.tick().await;
            if cancel_rx.try_recv().is_ok() {
                break;
            }

            let action = LightAction::Brightness {
                brightness: brightness as u8,
            };
            sel.execute(&stack, action.into()).unwrap(); //FIXME
            brightness += step_brightness;
        }

        // clean up
        let mut script_states = stack.script_states.lock().await;
        script_states.current.remove(&id);
    });

    Ok(())
}
