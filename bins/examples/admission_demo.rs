//! Admission-control demo for the arduous_rt integration.
//!
//! Two phases run the same workload — 50 "core" units (priority High),
//! 50 "crawler" units (priority Low, sheddable), and 50 "batch" units
//! (priority Medium) — through `arduous_rt::try_run`:
//!
//!   phase 1, Load::Normal:   everything completes
//!   phase 2, Load::High:     core completes, crawler sheds with
//!                            `AdmissionError::Shed`; batch survives
//!   phase 3, Load::Critical: core still completes; crawler and batch
//!                            shed — priority order in action
//!
//! Every shed decision lands in arduous_rt's audit ring; the demo
//! prints the trail at the end and exits non-zero if the outcome
//! differs from the expectation.
//!
//! Run with:
//!   cargo run -p gnostr-bins --example admission_demo

use arduous_rt::{
    audit_events, named_runtime, policy_from_toml, report_load, set_admission, set_policy, try_run,
    AdmissionConfig, AdmissionError, AuditKind, Load, Priority,
};

const POLICY: &str = r#"
deny = []

[[allow]]
pattern = "org.gnostr/**"
"#;

const TASKS_PER_DOMAIN: usize = 50;

async fn work() -> u64 {
    for _ in 0..3 {
        tokio::task::yield_now().await;
    }
    42
}

#[derive(Default)]
struct Counts {
    completed: usize,
    shed: usize,
    other: usize,
}

async fn drain(set: &mut tokio::task::JoinSet<Result<u64, AdmissionError>>) -> Counts {
    let mut counts = Counts::default();
    while let Some(result) = set.join_next().await {
        match result.expect("task join") {
            Ok(_) => counts.completed += 1,
            Err(AdmissionError::Shed { .. }) => counts.shed += 1,
            Err(_) => counts.other += 1,
        }
    }
    counts
}

#[derive(Default)]
struct PhaseCounts {
    core: Counts,
    crawler: Counts,
    batch: Counts,
}

async fn phase(load: Load) -> PhaseCounts {
    report_load(load);
    let core = named_runtime("org.gnostr/core");
    let crawler = named_runtime("org.gnostr/crawler");
    let batch = named_runtime("org.gnostr/batch");

    let mut core_set = tokio::task::JoinSet::new();
    let mut crawler_set = tokio::task::JoinSet::new();
    let mut batch_set = tokio::task::JoinSet::new();
    for _ in 0..TASKS_PER_DOMAIN {
        core_set.spawn(try_run(core, work()));
        crawler_set.spawn(try_run(crawler, work()));
        batch_set.spawn(try_run(batch, work()));
    }
    PhaseCounts {
        core: drain(&mut core_set).await,
        crawler: drain(&mut crawler_set).await,
        batch: drain(&mut batch_set).await,
    }
}

fn report(label: &str, counts: &PhaseCounts) {
    println!(
        "{label}: core {}/{} ({} shed), crawler {}/{} ({} shed), batch {}/{} ({} shed)",
        counts.core.completed,
        TASKS_PER_DOMAIN,
        counts.core.shed,
        counts.crawler.completed,
        TASKS_PER_DOMAIN,
        counts.crawler.shed,
        counts.batch.completed,
        TASKS_PER_DOMAIN,
        counts.batch.shed,
    );
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    set_policy(policy_from_toml(POLICY)?)?;
    set_admission(
        "org.gnostr/core",
        AdmissionConfig::default().with_priority(Priority::High),
    )?;
    set_admission(
        "org.gnostr/crawler",
        AdmissionConfig::default().with_priority(Priority::Low),
    )?;
    set_admission(
        "org.gnostr/batch",
        AdmissionConfig::default().with_priority(Priority::Medium),
    )?;

    let normal = phase(Load::Normal).await;
    report("normal load  ", &normal);

    let high = phase(Load::High).await;
    report("high load    ", &high);

    let critical = phase(Load::Critical).await;
    report("critical load", &critical);

    report_load(Load::Normal);

    println!("\naudit trail (shed events):");
    let mut printed = 0usize;
    for event in audit_events() {
        if matches!(event.kind, AuditKind::Shed | AuditKind::ShedCollapsed) {
            println!("  {event:?}");
            printed += 1;
            if printed >= 6 {
                println!("  ... ({} more)", audit_events().len() - 6);
                break;
            }
        }
    }
    if printed == 0 {
        println!("  (none)");
    }

    let ok = normal.core.completed == TASKS_PER_DOMAIN
        && normal.crawler.completed == TASKS_PER_DOMAIN
        && normal.batch.completed == TASKS_PER_DOMAIN
        && high.core.completed == TASKS_PER_DOMAIN
        && high.core.shed == 0
        && high.crawler.shed == TASKS_PER_DOMAIN
        && high.crawler.completed == 0
        && high.batch.completed == TASKS_PER_DOMAIN
        && critical.core.completed == TASKS_PER_DOMAIN
        && critical.crawler.shed == TASKS_PER_DOMAIN
        && critical.batch.shed == TASKS_PER_DOMAIN;
    if !ok {
        eprintln!("admission demo FAILED: outcome differs from expectation");
        std::process::exit(1);
    }
    println!("\nadmission demo OK: shedding followed the priority classes");
    Ok(())
}
