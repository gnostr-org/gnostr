// global_rt
//
// Delegates to arduous_rt's process-wide named registry: one shared,
// governed runtime per name instead of a copy-pasted OnceCell static.
//
// The governance policy is installed once, before the first runtime is
// created: a built-in default allows the org.gnostr domain, and
// GNOSTR_RUNTIME_POLICY can point at a TOML policy to replace it (see
// config/arduous_rt.toml for an example). gnostr-chat carries its own
// copy of ensure_policy because it cannot depend on the root crate;
// the policy is process-global in arduous_rt, so the content converges.
//
// Note: this runtime used to be current-thread; arduous_rt runtimes are
// multi-thread (the shared "org.gnostr/core" pool).

use std::sync::OnceLock;

/// Built-in fallback policy: allow the gnostr domain, deny nothing.
const DEFAULT_POLICY: &str = r#"
deny = []

[[allow]]
pattern = "org.gnostr/**"
"#;

static POLICY: OnceLock<()> = OnceLock::new();

/// Install the governance policy once per process.
pub(crate) fn ensure_policy() {
    POLICY.get_or_init(|| {
        let toml = std::env::var("GNOSTR_RUNTIME_POLICY")
            .ok()
            .and_then(|path| std::fs::read_to_string(path).ok())
            .unwrap_or_else(|| DEFAULT_POLICY.to_string());
        match arduous_rt::policy_from_toml(&toml) {
            Ok(policy) => {
                if let Err(err) = arduous_rt::set_policy(policy) {
                    tracing::warn!("arduous_rt policy rejected: {err}");
                }
            }
            Err(err) => tracing::warn!("arduous_rt policy invalid: {err}"),
        }
    });
}

pub fn global_rt() -> &'static tokio::runtime::Runtime {
    ensure_policy();
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
