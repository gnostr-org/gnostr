//! Admission-control demo for the arduous_rt integration.
//!
//! Two phases run the same workload — 50 "core" units (priority High)
//! and 50 "crawler" units (priority Low, sheddable) — through
//! `arduous_rt::try_run`:
//!
//!   phase 1, Load::Normal: everything completes
//!   phase 2, Load::High:   core completes, crawler sheds with
//!                          `AdmissionError::Shed`
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

async fn phase(load: Load) -> (Counts, Counts) {
    report_load(load);
    let core = named_runtime("org.gnostr/core");
    let crawler = named_runtime("org.gnostr/crawler");

    let mut core_set = tokio::task::JoinSet::new();
    let mut crawler_set = tokio::task::JoinSet::new();
    for _ in 0..TASKS_PER_DOMAIN {
        core_set.spawn(try_run(core, work()));
        crawler_set.spawn(try_run(crawler, work()));
    }
    let core_counts = drain(&mut core_set).await;
    let crawler_counts = drain(&mut crawler_set).await;
    (core_counts, crawler_counts)
}

fn report(label: &str, core: &Counts, crawler: &Counts) {
    println!(
        "{label}: core {}/{} completed ({} shed), crawler {}/{} completed ({} shed)",
        core.completed,
        TASKS_PER_DOMAIN,
        core.shed,
        crawler.completed,
        TASKS_PER_DOMAIN,
        crawler.shed,
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

    let (core_normal, crawler_normal) = phase(Load::Normal).await;
    report("normal load", &core_normal, &crawler_normal);

    let (core_high, crawler_high) = phase(Load::High).await;
    report("high load  ", &core_high, &crawler_high);

    report_load(Load::Normal);

    println!("\naudit trail (shed events):");
    let mut printed = 0usize;
    for event in audit_events() {
        if matches!(event.kind, AuditKind::Shed | AuditKind::ShedCollapsed) {
            println!("  {event:?}");
            printed += 1;
        }
    }
    if printed == 0 {
        println!("  (none)");
    }

    let ok = core_normal.completed == TASKS_PER_DOMAIN
        && crawler_normal.completed == TASKS_PER_DOMAIN
        && core_high.completed == TASKS_PER_DOMAIN
        && core_high.shed == 0
        && crawler_high.shed == TASKS_PER_DOMAIN
        && crawler_high.completed == 0;
    if !ok {
        eprintln!("admission demo FAILED: outcome differs from expectation");
        std::process::exit(1);
    }
    println!("\nadmission demo OK: shedding followed the priority classes");
    Ok(())
}
