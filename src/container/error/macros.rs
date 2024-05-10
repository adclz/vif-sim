#[macro_export]
macro_rules! error {
    ($message: expr, $location: expr, $trace: expr) => {
        $crate::container::error::error::Stop::new(
            $message,
            &Some($location),
            $trace
        )
    };
    ($message: expr, $location: expr) => {
        $crate::container::error::error::Stop::new(
            $message, 
            &Some($location),
            None
        )
    };
    ($message: expr) => {
        $crate::container::error::error::Stop::new(
            $message,  
            &None,
            None
        )
    };
}
