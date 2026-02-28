use std::collections::{HashMap, HashSet};

// --- 1. GIT-COMPLIANT SHA-1 ENGINE ---
fn git_sha1(data: &[u8]) -> String {
    let mut h0: u32 = 0x67452301;
    let mut h1: u32 = 0xEFCDAB89;
    let mut h2: u32 = 0x98BADCFE;
    let mut h3: u32 = 0x10325476;
    let mut h4: u32 = 0xC3D2E1F0;

    let mut padded = data.to_vec();
    let bit_len = (padded.len() as u64) * 8;
    padded.push(0x80);
    while (padded.len() * 8) % 512 != 448 { padded.push(0); }
    padded.extend_from_slice(&bit_len.to_be_bytes());

    for chunk in padded.chunks(64) {
        let mut w = [0u32; 80];
        for i in 0..16 {
            w[i] = u32::from_be_bytes([chunk[i*4], chunk[i*4+1], chunk[i*4+2], chunk[i*4+3]]);
        }
        for i in 16..80 { w[i] = (w[i-3] ^ w[i-8] ^ w[i-14] ^ w[i-16]).rotate_left(1); }
        let (mut a, mut b, mut c, mut d, mut e) = (h0, h1, h2, h3, h4);
        for i in 0..80 {
            let (f, k) = match i {
                0..=19 => ((b & c) | ((!b) & d), 0x5A827999),
                20..=39 => (b ^ c ^ d, 0x6ED9EBA1),
                40..=59 => ((b & c) | (b & d) | (c & d), 0x8F1BBCDC),
                _ => (b ^ c ^ d, 0xCA62C1D6),
            };
            let temp = a.rotate_left(5).wrapping_add(f).wrapping_add(e).wrapping_add(k).wrapping_add(w[i]);
            e = d; d = c; c = b.rotate_left(30); b = a; a = temp;
        }
        h0 = h0.wrapping_add(a); h1 = h1.wrapping_add(b);
        h2 = h2.wrapping_add(c); h3 = h3.wrapping_add(d); h4 = h4.wrapping_add(e);
    }
    format!("{:08x}{:08x}{:08x}{:08x}{:08x}", h0, h1, h2, h3, h4)
}

// --- 2. DATA STRUCTURES ---
#[derive(Clone, Debug)]
struct Commit {
    hash: String,
    parent_hash: Option<String>,
    message: String,
    snapshot: HashMap<String, String>,
}

pub struct MemoryRepo {
    history: HashMap<String, Commit>,
    heads: HashMap<String, String>,
    current_branch: String,
    working_directory: HashMap<String, String>,
}

impl MemoryRepo {
    pub fn new() -> Self {
        Self {
            history: HashMap::new(),
            heads: HashMap::new(),
            current_branch: "main".to_string(),
            working_directory: HashMap::new(),
        }
    }

    // --- 3. CORE ACTIONS ---
    pub fn write(&mut self, path: &str, content: &str) {
        self.working_directory.insert(path.to_string(), content.to_string());
    }

    fn calculate_tree_hash(&self) -> String {
        if self.working_directory.is_empty() {
            // Standard Git Empty Tree Hash: 4b825dc642cb6eb9a060e54bf8d69288fbee4904
            return git_sha1(b"tree 0\0");
        }
        let mut content = b"tree ".to_vec();
        let mut entries = String::new();
        let mut keys: Vec<_> = self.working_directory.keys().collect();
        keys.sort();
        for k in keys {
            entries.push_str(&format!("blob {}\0{}", self.working_directory[k].len(), self.working_directory[k]));
        }
        content.extend_from_slice(entries.len().to_string().as_bytes());
        content.push(0);
        content.extend_from_slice(entries.as_bytes());
        git_sha1(&content)
    }

    pub fn commit(&mut self, message: &str) {
        let tree_hash = self.calculate_tree_hash();
        let parent_hash = self.heads.get(&self.current_branch).cloned();
        
        let commit = Commit {
            hash: tree_hash.clone(),
            parent_hash,
            message: message.to_string(),
            snapshot: self.working_directory.clone(),
        };

        self.history.insert(tree_hash.clone(), commit);
        self.heads.insert(self.current_branch.clone(), tree_hash.clone());
        println!("[{}] Commit hash: {}", self.current_branch, tree_hash);
    }

    pub fn create_branch(&mut self, name: &str) {
        if let Some(tip) = self.heads.get(&self.current_branch).cloned() {
            self.heads.insert(name.to_string(), tip);
            self.current_branch = name.to_string();
            println!("--> Branch '{}' created.", name);
        }
    }

    pub fn switch_branch(&mut self, name: &str) {
        if let Some(hash) = self.heads.get(name).cloned() {
            self.current_branch = name.to_string();
            self.working_directory = self.history[&hash].snapshot.clone();
            println!("--> Switched to '{}'", name);
        }
    }

    // --- 4. DIFF ENGINE (LCS) ---
    pub fn diff_branch(&self, path: &str, branch_a: &str, branch_b: &str) {
        let h_a = self.heads.get(branch_a).expect("Missing branch A");
        let h_b = self.heads.get(branch_b).expect("Missing branch B");
        let c_a = self.history[h_a].snapshot.get(path).map(|s| s.as_str()).unwrap_or("");
        let c_b = self.history[h_b].snapshot.get(path).map(|s| s.as_str()).unwrap_or("");
        
        println!("\n--- Diff: {} ({} vs {}) ---", path, branch_a, branch_b);
        self.run_lcs(c_a, c_b);
    }

    fn run_lcs(&self, old_t: &str, new_t: &str) {
        let o: Vec<&str> = old_t.lines().collect();
        let n: Vec<&str> = new_t.lines().collect();
        let mut table = vec![vec![0; n.len() + 1]; o.len() + 1];
        for i in 1..=o.len() {
            for j in 1..=n.len() {
                table[i][j] = if o[i-1] == n[j-1] { table[i-1][j-1] + 1 } 
                else { std::cmp::max(table[i-1][j], table[i][j-1]) };
            }
        }
        self.backtrack(&table, &o, &n, o.len(), n.len());
    }

    fn backtrack(&self, t: &Vec<Vec<i32>>, o: &[&str], n: &[&str], i: usize, j: usize) {
        if i > 0 && j > 0 && o[i-1] == n[j-1] {
            self.backtrack(t, o, n, i-1, j-1);
            println!("  {}", o[i-1]);
        } else if j > 0 && (i == 0 || t[i][j-1] >= t[i-1][j]) {
            self.backtrack(t, o, n, i, j-1);
            println!("+ {}", n[j-1]);
        } else if i > 0 && (j == 0 || t[i][j-1] < t[i-1][j]) {
            self.backtrack(t, o, n, i-1, j);
            println!("- {}", o[i-1]);
        }
    }

    // --- 5. HISTORY ---
    pub fn log(&self) {
        println!("\n--- Log: {} ---", self.current_branch);
        let mut curr = self.heads.get(&self.current_branch).cloned();
        let mut seen = HashSet::new();
        while let Some(h) = curr {
            if !seen.insert(h.clone()) { break; }
            let c = &self.history[&h];
            println!("commit {}\nMessage: {}\n", c.hash, c.message);
            curr = c.parent_hash.clone();
        }
    }
}

fn main() {
    let mut repo = MemoryRepo::new();

    // 1. Create the Git-Standard empty commit
    repo.commit("Initial Empty Commit");

    // 2. Add files and branch
    repo.write("main.c", "int main() { return 0; }");
    repo.commit("Add code");
    
    repo.create_branch("dev");
    repo.write("main.c", "int main() { printf(\"BIP-64MOD\"); return 0; }");
    repo.commit("Add printf");

    // 3. Verify all features are present
    repo.log();
    repo.diff_branch("main.c", "main", "dev");
}
