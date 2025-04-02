// Error handling macros
// Provides macros for simplified error handling

/// Try to execute an expression that returns a Result, and convert the error
/// to a BoxError if it fails
#[macro_export]
macro_rules! try_box_result {
    ($expr:expr) => {
        match $expr {
            Ok(val) => val,
            Err(err) => return Err($crate::conversion::to_box_error(err)),
        }
    };
}

/// Create a new error with a message in the given domain
#[macro_export]
macro_rules! make_error {
    ($domain:expr, $code:expr, $message:expr) => {
        $crate::custom_error::CustomError::new($domain, $code, $message).into_box_error()
    };
}

/// Return early with an error if a condition is not satisfied
#[macro_export]
macro_rules! ensure {
    ($cond:expr, $domain:expr, $code:expr, $message:expr) => {
        if !($cond) {
            return Err($crate::make_error!($domain, $code, $message));
        }
    };
    ($cond:expr, $error:expr) => {
        if !($cond) {
            return Err($crate::conversion::to_box_error($error));
        }
    };
}

/// Bail early with an error
#[macro_export]
macro_rules! bail {
    ($domain:expr, $code:expr, $message:expr) => {
        return Err($crate::make_error!($domain, $code, $message));
    };
    ($error:expr) => {
        return Err($crate::conversion::to_box_error($error));
    };
}

/// Create a new custom error
#[macro_export]
macro_rules! custom_error {
    ($domain:expr, $code:expr, $message:expr) => {
        $crate::custom_error::CustomError::new($domain, $code, $message)
    };
}

/// Log an error and continue
#[macro_export]
macro_rules! log_error {
    ($result:expr) => {
        match $result {
            Ok(val) => val,
            Err(err) => {
                tracing::error!("Error: {}", err);
                continue;
            }
        }
    };
    ($result:expr, $message:expr) => {
        match $result {
            Ok(val) => val,
            Err(err) => {
                tracing::error!("{}: {}", $message, err);
                continue;
            }
        }
    };
}

/// Convert a Result to an Option, logging the error if it exists
#[macro_export]
macro_rules! result_to_option {
    ($result:expr) => {
        match $result {
            Ok(val) => Some(val),
            Err(err) => {
                tracing::error!("Error: {}", err);
                None
            }
        }
    };
    ($result:expr, $message:expr) => {
        match $result {
            Ok(val) => Some(val),
            Err(err) => {
                tracing::error!("{}: {}", $message, err);
                None
            }
        }
    };
} 