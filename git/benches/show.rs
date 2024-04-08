use criterion::{criterion_group, criterion_main, Criterion};
use gnostr_git::cli::Commands;
use gnostr_git::term::TermBackend;
use ratatui::backend::TestBackend;
use ratatui::Terminal;

fn show(c: &mut Criterion) {
    c.bench_function("show", |b| {
        let mut terminal = Terminal::new(TermBackend::Test(TestBackend::new(80, 1000))).unwrap();
        b.iter(|| {
            gnostr_git::run(
                &gnostr_git::cli::Args {
                    command: Some(Commands::Show {
                        reference: "f2137b4e5f6125b1097974c88e71d42ce29e0428".into(),
                    }),
                    print: true,
                    ..Default::default()
                },
                &mut terminal,
            )
            .unwrap();
        })
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default();
    targets = show
}
criterion_main!(benches);
