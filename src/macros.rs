#[macro_export]
macro_rules! on_option {
    ( $e:expr, $err_exp:expr ) => {
        match $e {
            Some(x) => x,
            None => { $err_exp },
        }
    }
}

/// Unwrap value. Return ```Ok(x)``` or performs action on error
///
/// Variable ```err``` represents error case
///
/// Example
/// ```ignore
/// let x = on_error!(f(x), return Err(err));
///
/// let x = match f(x) {
///    Ok(x) => x,
///    Err(err) => { return Err(err) },
/// };
/// ```
#[macro_export]
macro_rules! on_error {
    ( $e:expr, $err_exp:expr ) => {
        match $e {
            Ok(x) => x,
            #[allow(unused_variables)]
            Err(err) => { $err_exp },
        }
    }
}

#[macro_export]
macro_rules! on_error_ret {
    ( $e:expr, $err_exp:expr ) => {
        crate::on_error!($e, return Err($err_exp))
    }
}