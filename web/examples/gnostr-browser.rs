/*
 * Copyright (c) 2026 Geekspeaker Inc. All Rights Reserved.
 *
 * This software is "Source Available".
 * You may use and modify it for personal use.
 * Redistribution of modified versions is prohibited.
 *
 * See the LICENSE file for more details.
 */

use methodwise::run;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    run().await
}
