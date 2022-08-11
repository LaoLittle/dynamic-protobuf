#[macro_export]
macro_rules! dynamic_message {
    ($($k:expr => $v:expr),* $(,)?) => {
        {
            let mut m = $crate::DynamicMessage::new();
            $(m.set($k, $v);)*
            m
        }
    };
}
