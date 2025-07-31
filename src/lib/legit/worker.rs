use crypto::digest::Digest;
use crypto::sha1;
use log::debug;
use std::sync::mpsc;
//use time;

pub struct Worker {
    id: u32,
    digest: sha1::Sha1,
    tx: mpsc::Sender<(u32, String, String)>,
    target: String,
    tree: String,
    parent: String,
    author: String,
    repo: String,
    pwd_hash: String,
    message: String,
    timestamp: time::Tm,
    weeble: String,
    wobble: String,
    blockheight: String,
}

impl Worker {
    pub fn new(
        id: u32,
        //digest: sha1::Sha1,
        target: String,
        tree: String,
        parent: String,
        author: String,
        repo: String,
        pwd_hash: String,
        message: String,
        timestamp: time::Tm,
        weeble: String,
        wobble: String,
        blockheight: String,
        tx: mpsc::Sender<(u32, String, String)>,
    ) -> Worker {
        Worker {
            id,
            digest: sha1::Sha1::new(),
            target,
            tree,
            parent,
            author,
            repo,
            pwd_hash,
            message,
            timestamp,
            weeble,
            wobble,
            blockheight,
            tx,
        }
    }

    pub fn work(&mut self) {
        let tstamp = format!("{}", self.timestamp.strftime("%s %z").unwrap());

        let mut value = 0u32;
        loop {
            let (raw, blob) = self.generate_blob(value, &tstamp);
            let result = self.calculate(&blob);

            if result.starts_with(&self.target) {
                self.tx.send((self.id, raw, result));
                break;
            }

            value += 1;
        }
    }

    fn generate_blob(&mut self, value: u32, tstamp: &str) -> (String, String) {
        debug!("self.message={}\n", self.message);

        debug!("self.tree={}\n", self.tree);
        debug!("self.parent={}\n", self.parent);
        debug!("self.author={}\n", self.author);
        debug!("self.author={}\n", self.author);
        //debug!("self.committer={}\n",self.committer);
        debug!("self.tree={}\n", self.tree);
        debug!("self.parent={}\n", self.parent);
        debug!("self.weeble.trim()={}\n", self.weeble.trim());
        debug!("self.blockheight.trim()={}\n", self.blockheight.trim());
        debug!("self.wobble.trim()={}\n", self.wobble.trim());
        debug!("self.id={}\n", self.id);
        debug!("self.value={}\n", value);
        debug!("self.message={}\n", self.message);

        let raw = format!(
			"tree {}\n\
			parent {}\n\
			author {} {}\n\
			committer {} {}\n\n\
            {}/{}/{}:{}\n\n\"tree\":\"{}\",\"parent\":\"{}\",\"weeble\":\"{:04}\",\"blockheight\":\"{:06}\",\"wobble\":\"{:}\",\"bit\":\"{:02}\",\"nonce\":\"{:08x}\",\"message\":\"{:}\"",

            //below are in essential format
			self.tree,
			self.parent,
			self.author, tstamp, //author
			self.author, tstamp, //committer
            //above are in essential format

			//first element is commit subject line
			self.weeble.trim(),
			self.blockheight.trim(),
			self.wobble.trim(),
			self.message,

            //event body
			self.tree,
			self.parent,
			self.weeble.trim(),
			self.blockheight.trim(),
			self.wobble.trim(),
			self.id, value,
			self.message
		);
        debug!("raw={}\n", raw);

        //be careful when changing - fails silently when wrong.
        let blob = format!("commit {}\0{}", raw.len(), raw);
            debug!("blob={}\n", blob);

        (raw, blob)
    }

    fn calculate(&mut self, blob: &str) -> String {
        self.digest.reset();
        self.digest.input_str(blob);

        self.digest.result_str()
    }
}
