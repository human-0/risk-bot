pub mod spsa;

use std::collections::HashMap;

use risk_shared::player::PlayerBot;

pub trait CreateFromParams {
    type Bot: PlayerBot;

    fn create_from_params(&self, params: &HashMap<String, f64>) -> Self::Bot;
}

#[macro_export]
macro_rules! float_params {
    (
        $(
            ($_1:expr, $_2:expr, $_3:expr, $_4:expr) => {
                $($name:ident: $val:expr,)*
            }
        )*
    ) => {
        [
            $(
                $(
                    (stringify!($name), ($val, $_1, $_2, $_3, $_4)),
                )*
            )*
        ]
    };
}

#[macro_export]
macro_rules! eval_params {
    (
        $(
            { ($_11:expr, $_12:expr, $_13:expr, $_14:expr), ($_21:expr, $_22:expr, $_23:expr, $_24:expr) } => {
                $($name:ident: Eval($val1:expr, $val2:expr),)*
            }
        )*
    ) => {
        [
            $(
                $(
                    (concat!(stringify!($name), "_0"), ($val1, $_11, $_12, $_13, $_14)),
                    (concat!(stringify!($name), "_1"), ($val2, $_21, $_22, $_23, $_24)),
                )*
            )*
        ]
    };

    (
        $(
            ($_1:expr, $_2:expr, $_3:expr, $_4:expr) => {
                $($name:ident: Eval($val1:expr, $val2:expr),)*
            }
        )*
    ) => {
        [
            $(
                $(
                    (concat!(stringify!($name), "_0"), ($val1, $_1, $_2, $_3, $_4)),
                    (concat!(stringify!($name), "_1"), ($val2, $_1, $_2, $_3, $_4)),
                )*
            )*
        ]
    };
}
