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
        apply_runtime_shaping();
        spawn_audit_relay();
    });
}

/// Runtime shaping applied alongside the policy: gnostr-chat's libp2p
/// tasks ride a small dedicated pool, and the whole domain carries a
/// thread budget as a safety rail. Kept in code because arduous_rt's
/// `RuntimeConfig` is imperative — the TOML policy covers
/// allow/deny/quota only.
fn apply_runtime_shaping() {
    let _ = arduous_rt::allow_configured(
        "org.gnostr/chat",
        arduous_rt::RuntimeConfig::default()
            .with_worker_threads(4)
            .with_thread_name_prefix("gnostr-chat"),
    );
    // The thread budget is a production safety rail; unit tests run
    // hundreds of threads across parallel test runtimes, so it only
    // arms outside test builds.
    #[cfg(not(test))]
    let _ = arduous_rt::set_thread_quota("org.gnostr/**", 32);
}

/// With GNOSTR_AUDIT_RELAY set, forward arduous_rt governance events to
/// tracing::debug from a background thread. PoC-grade wiring — a
/// production integration would drain at explicit points or bridge via a
/// tracing layer.
fn spawn_audit_relay() {
    if std::env::var_os("GNOSTR_AUDIT_RELAY").is_none() {
        return;
    }
    std::thread::spawn(|| {
        let mut seen = 0usize;
        loop {
            let events = arduous_rt::audit_events();
            if events.len() < seen {
                seen = 0; // ring buffer wrapped; resync
            }
            for event in events.iter().skip(seen) {
                tracing::debug!(?event, "arduous_rt governance event");
            }
            seen = events.len();
            std::thread::sleep(std::time::Duration::from_secs(2));
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

    #[test]
    fn test_policy_governs_outside_the_gnostr_domain() {
        global_rt(); // install the policy first

        // Names outside the org.gnostr domain are rejected.
        assert!(matches!(
            arduous_rt::try_named_runtime("com.example.intruder"),
            Err(arduous_rt::NameError::NotAllowed { .. })
        ));

        // Subtree names under the gnostr domain stay allowed.
        assert!(arduous_rt::try_named_runtime("org.gnostr/test-guard").is_ok());
    }
}
