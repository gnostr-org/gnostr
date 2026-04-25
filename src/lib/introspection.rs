#[macro_export]
macro_rules! introspection_here {
    ($message:expr) => {
        format!("{}::{} {}", module_path!(), function_name!(), $message)
    };
}

#[macro_export]
macro_rules! introspection_debug {
    ($message:expr) => {
        format!("[DEBUG] {}::{} {}", module_path!(), function_name!(), $message)
    };
}

#[cfg(test)]
mod tests {
    #[function_name::named]
    #[test]
    fn formats_function_context() {
        let message = crate::introspection_here!("ready");
        assert_eq!(
            message,
            format!("{}::{} ready", module_path!(), function_name!())
        );
    }

    #[function_name::named]
    #[test]
    fn formats_debug_context() {
        let message = crate::introspection_debug!("ready");
        assert_eq!(
            message,
            format!("[DEBUG] {}::{} ready", module_path!(), function_name!())
        );
    }
}
