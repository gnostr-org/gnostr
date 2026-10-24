use crypto::digest::Digest;
use crypto::sha1;
use std::sync::mpsc;
use time_0_3::format_description;
use time_0_3::OffsetDateTime;

pub struct Worker {
    id: u32,
    digest: sha1::Sha1,
    tx: mpsc::Sender<(u32, String, String)>,
    target: String,
    tree: String,
    parent: String,
    author: String,
    message: String,
    timestamp: OffsetDateTime,
}

impl Worker {
    pub fn new(
        id: u32,
        target: String,
        tree: String,
        parent: String,
        author: String,
        message: String,
        timestamp: OffsetDateTime,
        tx: mpsc::Sender<(u32, String, String)>,
    ) -> Worker {
        debug!(
            "[Worker {}] Initializing: target={}, tree={}, parent={}, message bytes={}, timestamp={}",
            id,
            target,
            tree,
            parent,
            message.len(),
            timestamp
        );
        Worker {
            id,
            digest: sha1::Sha1::new(),
            tx,
            target,
            tree,
            parent,
            author,
            message,
            timestamp,
        }
    }

    pub fn work(&mut self) {
        info!("[Worker {}] Starting work...", self.id);
        let format = format_description::parse(
            "[unix_timestamp] [offset_hour sign:mandatory][offset_minute]",
        )
        .unwrap();
        let tstamp = self.timestamp.format(&format).unwrap();
        debug!("[Worker {}] Generated timestamp: {}", self.id, tstamp);

        let mut value = 0u32;
        loop {
            if value % 50000 == 0 {
                debug!(
                    "[Worker {}] Still mining; iteration value {} for target {}",
                    self.id, value, self.target
                );
            }
            let (raw, blob) = self.generate_blob(value, &tstamp);
            let result = self.calculate(&blob);

            if result.starts_with(&self.target) {
                info!(
                    "[Worker {}] Target hash found! Hash: {}, Value: {}",
                    self.id, result, value
                );
                match self.tx.send((self.id, raw, result)) {
                    Ok(()) => debug!("[Worker {}] Sent mined commit to coordinator", self.id),
                    Err(err) => debug!(
                        "[Worker {}] Coordinator disconnected before receiving mined commit: {}",
                        self.id, err
                    ),
                }
                break;
            }

            value += 1;
        }
        debug!(
            "[Worker {}] Stopped after finding a matching commit",
            self.id
        );
    }

    fn generate_blob(&mut self, value: u32, tstamp: &str) -> (String, String) {
        let raw = format!(
            "tree {}\n\
                           parent {}\n\
                           author {} {}\n\
                           committer {} {}\n\n\
                           {}\n\n{:02}-{:08x}",
            self.tree,
            self.parent,
            self.author,
            tstamp,
            self.author,
            tstamp,
            self.message,
            self.id,
            value
        );
        let blob = format!("commit {}\0{}", raw.len(), raw);
        trace!(
            "[Worker {}] Candidate {}: raw bytes={}, blob bytes={}",
            self.id,
            value,
            raw.len(),
            blob.len()
        );
        (raw, blob)
    }

    fn calculate(&mut self, blob: &str) -> String {
        self.digest.reset();
        self.digest.input_str(blob);

        let hash = self.digest.result_str();
        trace!("[Worker {}] Candidate hash: {}", self.id, hash);
        hash
    }
}
