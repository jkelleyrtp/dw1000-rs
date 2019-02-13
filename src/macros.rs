//! Contains some macros that are useful for working with this crate


/// Blocks on a non-blocking operation until a timer times out
///
/// Expects two arguments: A timer, and an expression that evaluates to
/// `nb::Result<T, E>` and returns `Result<T, TimeoutError<E>>`.
#[macro_export]
macro_rules! block_timeout {
    ($timer:expr, $op:expr) => {
        {
            // Make sure the timer has the right type. If it isn't, the user
            // should at least get a good error message.
            fn check_type<T>(_: &mut T)
                where T: embedded_hal::timer::CountDown {}
            check_type($timer);

            loop {
                match $timer.wait() {
                    Ok(()) =>
                        break Err($crate::macros::TimeoutError::Timeout),
                    Err(nb::Error::WouldBlock) =>
                        (),
                    Err(_) =>
                        unreachable!(),
                }

                match $op {
                    Ok(result) =>
                        break Ok(result),
                    Err(nb::Error::WouldBlock) =>
                        (),
                    Err(nb::Error::Other(error)) =>
                        break Err($crate::macros::TimeoutError::Other(error)),
                }
            }
        }
    }
}

/// Repeats an operation until a timer times out
///
/// Expects four arguments:
/// - A timer
/// - An expression that evaluates to `Result<T, E>` (the operation)
/// - A closure that will be called every time the operation succeeds
/// - A closure that will be called every time the operation fails
///
/// This will keep repeating the operation until the timer runs out, no matter
/// whether it suceeds or fails.
#[macro_export]
macro_rules! repeat_timeout {
    ($timer:expr, $op:expr, $on_success:expr, $on_error:expr,) => {
        {
            // Make sure the timer has the right type. If it isn't, the user
            // should at least get a good error message.
            fn check_type<T>(_: &mut T)
                where T: embedded_hal::timer::CountDown {}
            check_type($timer);

            loop {
                match $timer.wait() {
                    Ok(()) =>
                        break,
                    Err(nb::Error::WouldBlock) =>
                        (),
                    Err(_) =>
                        unreachable!(),
                }

                match $op {
                    Ok(result) => {
                        $on_success(result);
                    }
                    Err(error) => {
                        $on_error(error);
                    }
                }
            }
        }
    }
}


/// An error that can be a timeout or another error
#[derive(Debug)]
pub enum TimeoutError<T> {
    /// The operation timed out
    Timeout,

    /// Another error occured
    Other(T),
}
