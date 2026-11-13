// global_rt
//
// Delegates to arduous_rt's process-wide named registry: one shared,
// governed runtime per name instead of a copy-pasted OnceCell static.
// Note: this runtime used to be current-thread; arduous_rt runtimes are
// multi-thread (the shared "gnostr.core" pool).
pub fn global_rt() -> &'static tokio::runtime::Runtime {
    arduous_rt::named_runtime("org.gnostr/core")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_global_rt() {
        let rt1 = global_rt();
        let rt2 = global_rt();

        // Ensure that the same runtime is returned each time.
        assert!(std::ptr::eq(rt1, rt2));

        // Ensure the runtime is functional by spawning a simple task.
        rt1.block_on(async {
            let result = tokio::spawn(async { 1 + 1 }).await.unwrap();
            assert_eq!(result, 2);
        });
    }
}
