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
/// Example
/// ```
/// use libvktypes::on_error;
///
/// // Two functions are identical
/// fn foo() -> Result<u32, &'static str> {
///     let x: Result<u32, &'static str> = Ok(42);
///
///     let result = match x {
///         Ok(val) => val,
///         Err(err) => { return Err("Foo error") },
///     };
///
///     Ok(result)
/// }
///
/// fn foo_with_macros() -> Result<u32, &'static str> {
///     let x: Result<u32, &'static str> = Ok(42);
///
///     let result = on_error!(x, return Err("Foo error"));
///
///     Ok(result)
/// }
/// ```
#[macro_export]
macro_rules! on_error {
    ( $e:expr, $err_exp:expr ) => {
        match $e {
            Ok(x) => x,
            Err(_) => { $err_exp },
        }
    }
}

#[macro_export]
macro_rules! on_error_ret {
    ( $e:expr, $err_exp:expr ) => {
        $crate::on_error!($e, return Err($err_exp))
    }
}

/// Return [`const pointer`](pointer) to collection's data
///
/// or [`null`](std::ptr::null) if there are no items in collection
///
/// Example
/// ```
/// use libvktypes::data_ptr;
///
/// let data = vec![42_u32];
///
/// // This expression
///
/// let data_ptr: *const u32 = data_ptr!(data);
///
/// // is equivalent to
///
/// let data_ptr: *const u32 = if data.is_empty() {
///     std::ptr::null()
/// }  else {
///     data.as_ptr()
/// };
/// ```
#[macro_export]
macro_rules! data_ptr {
    ( $e:expr ) => {
        if $e.is_empty() {
            std::ptr::null()
        } else {
            $e.as_ptr()
        }
    }
}