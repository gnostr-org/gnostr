use std::sync::mpsc;
use crypto::digest::Digest;
use crypto::sha1;
use time_0_3::OffsetDateTime;
use time_0_3::format_description;

pub struct Worker {
    id:      u32,
    digest:  sha1::Sha1,
    tx:      mpsc::Sender<(u32, String, String)>,
    target:  String,
    tree:    String,
    parent:  String,
    author:  String,
    message: String,
    timestamp: OffsetDateTime,
}

impl Worker {
    pub fn new(id:        u32,
               target:    String,
               tree:      String,
               parent:    String,
               author:    String,
               message:   String,
               timestamp: OffsetDateTime,
               tx:        mpsc::Sender<(u32, String, String)>) -> Worker {
        Worker {
            id:        id,
            digest:    sha1::Sha1::new(),
            tx:        tx,
            target:    target,
            tree:      tree,
            parent:    parent,
            author:    author,
            message:   message,
            timestamp: timestamp
        }
    }


    pub fn work(&mut self) {
        info!("[Worker {}] Starting work...", self.id);
        let format = format_description::parse("[unix_timestamp] [offset_hour sign:mandatory][offset_minute]").unwrap();
        let tstamp = self.timestamp.format(&format).unwrap();
        debug!("[Worker {}] Generated timestamp: {}", self.id, tstamp);

        let mut value  = 0u32;
        loop {
            if value % 100000 == 0 {
                debug!("[Worker {}] Current iteration value: {}", self.id, value);
            }
            let (raw, blob) = self.generate_blob(value, &tstamp);
            let result = self.calculate(&blob);

            if result.starts_with(&self.target) {
                info!("[Worker {}] Target hash found! Hash: {}, Value: {}", self.id, result, value);
                let _ = self.tx.send((self.id, raw, result));
                break;
            }

            value += 1;
        }
    }

    fn generate_blob(&mut self, value: u32, tstamp: &str) -> (String, String) {
        let raw = format!("tree {}\n\
                           parent {}\n\
                           author {} {}\n\
                           committer {} {}\n\n\
                           {}\n{:02}-{:08x}",
                          self.tree,
                          self.parent,
                          self.author, tstamp,
                          self.author, tstamp,
                          self.message,
                          self.id,
                          value);
        let blob = format!("commit {}\0{}", raw.len(), raw);

        (raw, blob)
    }

    fn calculate(&mut self, blob: &str) -> String {
        self.digest.reset();
        self.digest.input_str(blob);

        self.digest.result_str()
    }
}
