use std::sync::mpsc;

use crypto::digest::Digest;
use crypto::sha1;
//use time;

pub struct Worker {
    id: u32,
    digest: sha1::Sha1,
    tx: mpsc::Sender<(u32, String, String)>,
    target: String,
    tree: String,
    parent: String,
    author: String,
    _repo: String,
    _pwd_hash: String,
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
        _repo: String,
        _pwd_hash: String,
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
            _repo,
            _pwd_hash,
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
                let _ = self.tx.send((self.id, raw, result));
                break;
            }

            value += 1;
        }
    }

    fn generate_blob(&mut self, value: u32, tstamp: &str) -> (String, String) {
        if cfg!(debug_assertions) {
            print!("self.message={}\n", self.message);

            print!("self.tree={}\n", self.tree);
            print!("self.parent={}\n", self.parent);
            print!("self.author={}\n", self.author);
            print!("self.author={}\n", self.author);
            //print!("self.committer={}\n",self.committer);
            print!("self.tree={}\n", self.tree);
            print!("self.parent={}\n", self.parent);
            print!("self.weeble.trim()={}\n", self.weeble.trim());
            print!("self.blockheight.trim()={}\n", self.blockheight.trim());
            print!("self.wobble.trim()={}\n", self.wobble.trim());
            print!("self.id={}\n", self.id);
            print!("self.value={}\n", value);
            print!("self.message={}\n", self.message);
        }

        let raw = format!(
            "tree {}\nparent {}\nauthor {} {}\ncommitter {} \
             {}\n\n{}/{}/{}:{}\n\n\"tree\":\"{}\",\"parent\":\"{}\",\"weeble\":\"{:04}\",\"\
             blockheight\":\"{:06}\",\"wobble\":\"{:}\",\"bit\":\"{:02}\",\"nonce\":\"{:08x}\",\"\
             message\":\"{:}\"",
            //below are in essential format
            self.tree,
            self.parent,
            self.author,
            tstamp, //author
            self.author,
            tstamp, //committer
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
            self.id,
            value,
            self.message
        );
        if cfg!(debug_assertions) {
            print!("raw={}\n", raw);
        }

        //be careful when changing - fails silently when wrong.
        let blob = format!("commit {}\0{}", raw.len(), raw);
        if cfg!(debug_assertions) {
            print!("blob={}\n", blob);
        }

        (raw, blob)
    }

    fn calculate(&mut self, blob: &str) -> String {
        self.digest.reset();
        self.digest.input_str(blob);

        self.digest.result_str()
    }
}
