# [gnostr-legit](https://github.com/gnostr-org/gnostr-legit.git) [![gnostr-legit](https://github.com/gnostr-org/gnostr-legit/actions/workflows/automate.yml/badge.svg)](https://github.com/gnostr-org/gnostr-legit/actions/workflows/automate.yml)

##### gnostr-legit adds Proof of Work (PoW) to a git commit hash prefix.

###### gnostr-legit is part of the *[gnostr.org](https://gnostr.org)* command line utility suite. 

---

#### install rustup:

```
curl -sSf https://static.rust-lang.org/rustup.sh | sh
```

#### cargo:

```
cargo install gnostr-legit
```

example:

```
gnostr-legit . -p 00000 -m "gnostr-legit commit"
```
