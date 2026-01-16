use colored::{ColoredString, Colorize};
use russh::{server::Handle, ChannelId, CryptoVec};

use crate::utils::CustomContext;

use super::commands::Knob;

async fn send_message(
    handle: Handle,
    channel: ChannelId,
    title: ColoredString,
    message: &str,
) -> anyhow::Result<()> {
    let text = format!(
        "{}{}{} {}\n",
        "[".bold(),
        title.clone().bold(),
        "]".bold(),
        textwrap::wrap(message, 40).join("\n")
    );

    let text = CryptoVec::from(text);
    handle
        .extended_data(channel, 1, text)
        .await
        .context("Failed to send message over wire")?;
    Ok(())
}

impl Knob {
    pub async fn info(&self, message: &str) -> anyhow::Result<()> {
        send_message(self.handle.clone(), self.channel, "GNOSTR".green(), message).await
    }

    pub async fn error(&self, message: &str) -> anyhow::Result<()> {
        send_message(self.handle.clone(), self.channel, "GNOSTR".red(), message).await
    }

    pub async fn repo_note(&self, message: &str) -> anyhow::Result<()> {
        send_message(
            self.handle.clone(),
            self.channel,
            "REPO NOTE".yellow(),
            message,
        )
        .await
    }
}
