use std::{sync::OnceLock, collections::HashMap, error::Error, sync::Arc};

use tokio::sync::oneshot;

use crate::{entity::EntityType, state::IglooState};
use super::ScriptMeta;

// lowkey getting the hang of rust macros
macro_rules! gen_builtin_script_data {
    (
        $($script_name:ident(
            claims: {
                $($entity_type:ident: [$($sel_str:expr),*]),*
            },
            auto_cancel: $auto_cancel:expr,
            auto_run: $auto_run:expr
        )),*
    ) => {
        $(
            pub mod $script_name;
        )*

        static CLAIMS: OnceLock<HashMap<String, HashMap<EntityType, Vec<String>>>> = OnceLock::new();

        pub fn get_claims() -> &'static HashMap<String, HashMap<EntityType, Vec<String>>> {
            CLAIMS.get_or_init(|| {
                let mut map = HashMap::new();
                $(
                    let mut claims = HashMap::new();
                    $(
                        claims.insert(EntityType::$entity_type, vec![
                            $($sel_str.to_string(),)*
                        ]);
                    )*
                    map.insert(stringify!($script_name).to_string(), claims);
                )*
                map
            })
        }

        pub fn get_meta(name: &str) -> Option<ScriptMeta> {
            match name {
                $(
                    stringify!($script_name) => Some(ScriptMeta {
                        claims: get_claims().get(stringify!($script_name)).unwrap(),
                        auto_cancel: $auto_cancel,
                        auto_run: $auto_run,
                    }),
                )*
                _ => None
            }
        }

        pub async fn spawn(
            script_name: &str,
            id: u32,
            state: Arc<IglooState>,
            uid: Option<usize>,
            args: Vec<String>,
            cancel_rx: oneshot::Receiver<()>,
        ) -> Result<(), Box<dyn Error>> {
            match script_name {
                $(
                    stringify!($script_name) => $script_name::spawn(id, state, uid, args, cancel_rx).await?,
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
        auto_cancel: true,
        auto_run: false
    ),
    brightness_step(
        claims: {
            Light: ["$1"]
        },
        auto_cancel: true,
        auto_run: false
    )
);
