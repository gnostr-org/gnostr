# [Legit](https://github.com/RandyMcMillan/legit.git) [![legit](https://github.com/RandyMcMillan/legit/actions/workflows/automate.yml/badge.svg)](https://github.com/RandyMcMillan/legit/actions/workflows/automate.yml)

##### Legit adds Proof of Work (PoW) to a git commit hash prefix.


### `Example`

---

```
git log | grep "0000006"
```

`commit` [000000615b90566ae8559dd45852190edea79a8c](https://github.com/RandyMcMillan/legit/commit/000000615b90566ae8559dd45852190edea79a8c)

`commit` [000000615b90566ae8559dd45852190edea79a8c](https://github.com/RandyMcMillan/legit/commit/000000615b90566ae8559dd45852190edea79a8c)

---
### `Install`
---

#### `git:`

```shell
git clone https://github.com/RandyMcMillan/legit.git && \
cd legit && make legit
```

```
example:
```

```
example:
```

### `Usage`

---

```shell
git clone https://github.com/RandyMcMillan/legit.git
```

```shell
cd legit && make legit
```

#### `add files to commit`

```
git add <FILE_NAME> <FILE_NAME>
```

```
git stash (--include-untracked)
```

#### `legit command`
---

```
legit . -p "000000" -m "my legit commit"
```

```
git stash pop
```

--