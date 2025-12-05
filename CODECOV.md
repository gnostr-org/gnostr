# Rust Code Coverage

This document describes how to generate code coverage reports for this project without relying on third-party services.

## Prerequisites

Before you can generate coverage reports, you need to install the necessary LLVM tools. You can do this by adding the `llvm-tools-preview` component using `rustup`:

```bash
rustup component add llvm-tools-preview
```

## 1. Enable Coverage Instrumentation

Set the `RUSTFLAGS` environment variable to enable coverage instrumentation during the build process.

```bash
RUSTFLAGS="-C instrument-coverage" cargo build
```

## 2. Generate Raw Coverage Data

Run the test suite to generate the raw coverage data. This will create a `default.profraw` file in the project's root directory.

```bash
cargo test
```

## 3. Process Raw Data

Use the `llvm-profdata` tool, which is part of the LLVM toolchain included with Rust, to merge and process the raw data file.

```bash
grcov . --binary-path ./target/debug/ -s . -t html --branch --ignore-not-existing -o ./target/debug/coverage/
```

## 4. Generate Reports

Use the `llvm-cov` tool to generate human-readable reports from the processed data. You can either view a summary in the terminal or generate a detailed HTML report.

### Terminal Summary

```bash
llvm-cov report --instr-profile=default.profdata target/debug/<your-binary-name>
```

### HTML Report

This will generate a detailed, file-by-file HTML report in a directory named `coverage/`.

```bash
llvm-cov show --instr-profile=default.profdata target/debug/<your-binary-name> -format=html -output-dir=coverage
```
