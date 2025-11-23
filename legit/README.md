# [Legit](https://github.com/RandyMcMillan/legit.git) [![legit](https://github.com/RandyMcMillan/legit/actions/workflows/automate.yml/badge.svg)](https://github.com/RandyMcMillan/legit/actions/workflows/automate.yml)

#### Legit adds Proof of Work (PoW) to a git commit hash prefix.

#### install rustup:

```
curl -sSf https://static.rust-lang.org/rustup.sh | sh
```

#### cargo:

```
cargo install legit
```

#### `Example`

```
git log | grep "0000006"
```

`commit` [000000615b90566ae8559dd45852190edea79a8c](httpshttps://github.com/RandyMcMillan/legit/commit/000000615b90566ae8559dd45852190edea79a8c)



---



## Usage



To create a commit with a subject and a multi-line body, use the `-m` flag multiple times. The first instance of `-m` will be the commit subject, and each subsequent instance will be a new line in the commit body.



### Command Syntax



```bash

gnostr legit -m "<subject>" -m "<body_line_1>" -m "<body_line_2>" ...

```



### Example



This is the command used to successfully create a commit:



```bash

cargo run --bin gnostr -- legit \

  -m "fix(legit): improve error handling and argument parsing" \

  -m "Replaced panics with graceful error handling in gnostr-legit." \

  -m "Improved directory creation logic to prevent errors." \

  -m "Enabled multi-line commit messages with multiple -m flags."

```



This resulted in the following commit message:



```

fix(legit): improve error handling and argument parsing



Replaced panics with graceful error handling in gnostr-legit.

Improved directory creation logic to prevent errors.

Enabled multi-line commit messages with multiple -m flags.

```



### Important Note



The command-line parser currently has a limitation: **body lines cannot begin with a hyphen (`-`)**. If a line starts with a hyphen, the parser will mistakenly interpret it as a command-line flag and the command will fail.


