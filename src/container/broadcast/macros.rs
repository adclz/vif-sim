#[macro_export]
macro_rules! channel_msg {
    ($channel: expr, $msg: expr) => {
        $channel.add_message($msg);
    };
    ($channel: expr, $($msg: tt)*) => {
        $channel.add_message(format!($($msg)*).as_str())
    };
}
