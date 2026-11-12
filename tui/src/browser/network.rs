/*
 * Copyright (c) 2026 Geekspeaker Inc. All Rights Reserved.
 *
 * This software is "Source Available".
 * You may use and modify it for personal use.
 * Redistribution of modified versions is prohibited.
 *
 * See the LICENSE file for more details.
 */

use anyhow::{Context, Result};

// Mimic a modern browser to prevent blocking (Updated for 2026)
const USER_AGENT_STRING: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/145.0.0.0 Safari/537.36";

pub struct NetworkClient {
    client: reqwest::Client,
}

impl NetworkClient {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .user_agent(USER_AGENT_STRING)
            .redirect(reqwest::redirect::Policy::limited(10))
            // Sites like Amazon/Gmail expect standard headers
            .default_headers({
                let mut headers = reqwest::header::HeaderMap::new();
                headers.insert("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7".parse().unwrap());
                headers.insert("Accept-Language", "en-US,en;q=0.9".parse().unwrap());
                // Crucial: We enable compression now.
                headers.insert("Accept-Encoding", "gzip, deflate, br".parse().unwrap());
                // Anti-Bot Headers (Mimic Chrome)
                headers.insert("Upgrade-Insecure-Requests", "1".parse().unwrap());
                headers.insert("Sec-Fetch-Dest", "document".parse().unwrap());
                headers.insert("Sec-Fetch-Mode", "navigate".parse().unwrap());
                headers.insert("Sec-Fetch-Site", "none".parse().unwrap());
                headers.insert("Sec-Fetch-User", "?1".parse().unwrap());
                headers.insert("Connection", "keep-alive".parse().unwrap());
                headers
            })
            .build()
            .unwrap_or_default();
        Self { client }
    }

    pub async fn fetch_url(&self, url: &str) -> Result<(String, String)> {
        // Clean URL of any surrounding quotes first
        let clean_url = url
            .trim()
            .trim_matches('"')
            .trim_matches('\'')
            .replace("\"", "");

        // Ensure URL has scheme
        let target_url = if !clean_url.starts_with("http://") && !clean_url.starts_with("https://")
        {
            format!("https://{}", clean_url)
        } else {
            clean_url.to_string()
        };

        eprintln!("NET send start {}", target_url);
        let response = self
            .client
            .get(&target_url)
            .send()
            .await
            .context("Failed to send request")?;
        eprintln!("NET send done status={} url={}", response.status(), response.url());

        let final_url = response.url().to_string();
        let status = response.status();

        if !status.is_success() {
            return Err(anyhow::anyhow!("HTTP Error: {}", status));
        }

        eprintln!("NET text start");
        let content = response
            .text()
            .await
            .context("Failed to read response body")?;
        eprintln!("NET text done len={}", content.len());

        Ok((final_url, content))
    }
}
