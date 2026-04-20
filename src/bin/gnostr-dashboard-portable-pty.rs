use clap::Parser;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let raw_args: Vec<String> = std::env::args().collect();
    let mut commands = Vec::new();
    let mut filtered_args = vec![raw_args[0].clone()];

    // Manually parse for --command to handle unquoted multi-word commands
    // and remove them from args passed to clap.
    let mut i = 1;
    while i < raw_args.len() {
        if raw_args[i] == "--command" {
            i += 1;
            let mut current_cmd_parts = Vec::new();
            // Collect all following arguments until the next flag
            while i < raw_args.len() && !raw_args[i].starts_with("--") {
                current_cmd_parts.push(raw_args[i].clone());
                i += 1;
            }
            if !current_cmd_parts.is_empty() {
                commands.push(current_cmd_parts.join(" "));
            }
        } else {
            filtered_args.push(raw_args[i].clone());
            i += 1;
        }
    }

    // Use clap for other potential global flags (like --info, --debug, etc)
    let args = match gnostr::cli::GnostrCli::try_parse_from(filtered_args) {
        Ok(a) => a,
        Err(e) => {
            if e.kind() == clap::error::ErrorKind::DisplayHelp || e.kind() == clap::error::ErrorKind::DisplayVersion {
                e.exit();
            }
            // Fallback to default if parsing fails due to unrecognized dashboard-specific args
            gnostr::cli::GnostrCli::default()
        }
    };
    
    // Combine commands: prioritizes manual parsing, fallbacks to clap-parsed if any missed
    let mut final_commands = commands;
    if final_commands.is_empty() && !args.commands.is_empty() {
        final_commands = args.commands;
    }

    gnostr::dashboard::run_dashboard(final_commands).await
}
