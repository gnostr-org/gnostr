### To bypass compiling rust with formula cmake:


brew install --cask cmake
brew edit rust
comment out depends_on cmake and openssl@3 lines
brew install rustup-init


TODO: figure out ow to autmate this
