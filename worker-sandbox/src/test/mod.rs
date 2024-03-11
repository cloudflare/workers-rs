pub mod durable;
pub mod export_durable_object;
pub mod put_raw;

#[macro_export]
macro_rules! ensure {
    ($ex:expr, $er:expr) => {
        if !$ex {
            return Err($er.into());
        }
    };
}
