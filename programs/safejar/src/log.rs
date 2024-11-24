#[warn(unused_imports)]
use anchor_lang;

#[macro_export]
macro_rules! nplog {
    ($($rest:tt)*) => {
        #[cfg(feature="verbose")]
        anchor_lang::prelude::msg!($($rest)*);
        #[cfg(feature="test")]
        println!($($rest)*);
    };
}
