use std::{sync::OnceLock, collections::HashMap, error::Error, sync::Arc};

use tokio::sync::oneshot;

use crate::{command::SubdeviceType, map::IglooStack};
use super::ScriptMeta;

// lowkey getting the hang of rust macros
macro_rules! gen_builtin_script_data {
    (
        $($script_name:ident(
            claims: {
                $($subdev_type:ident: [$($sel_str:expr),*]),*
            },
            auto_cancel: $auto_cancel:expr
        )),*
    ) => {
        $(
            pub mod $script_name;
        )*

        static META: OnceLock<HashMap<String, ScriptMeta>> = OnceLock::new();

        pub fn get_meta() -> &'static HashMap<String, ScriptMeta> {
            META.get_or_init(|| {
                let mut map = HashMap::new();
                $(
                    let mut claims = HashMap::new();
                    $(
                        claims.insert(SubdeviceType::$subdev_type, vec![
                            $($sel_str.to_string(),)*
                        ]);
                    )*
                    let config = ScriptMeta {
                        claims,
                        auto_cancel: $auto_cancel
                    };
                    map.insert(stringify!($script_name).to_string(), config);
                )*
                map
            })
        }

        pub async fn spawn(
            script_name: &str,
            id: u32,
            stack: Arc<IglooStack>,
            uid: usize,
            args: Vec<String>,
            cancel_rx: oneshot::Receiver<()>,
        ) -> Result<(), Box<dyn Error>> {
            match script_name {
                $(
                    stringify!($script_name) => $script_name::spawn(id, stack, uid, args, cancel_rx).await?,
                )*
                _ => panic!("Mismatched builtin script run. THIS SHOULD NEVER HAPPEN")
            }
            Ok(())
        }
    }
}

gen_builtin_script_data!(
    rainbow(
        claims: {
            Light: ["$1"]
        },
        auto_cancel: true
    ),
    brightness_step(
        claims: {
            Light: ["$1"]
        },
        auto_cancel: true
    )
);
