use once_cell::sync::OnceCell;

// global_rt
pub fn global_rt() -> &'static tokio::runtime::Runtime {
    static RT: OnceCell<tokio::runtime::Runtime> = OnceCell::new();
    RT.get_or_init(|| tokio::runtime::Runtime::new().unwrap())
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
