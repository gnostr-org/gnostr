FROM rust:latest AS build
WORKDIR /app
RUN apt-get update && apt-get install -y \
    curl \
    sudo \
    tar \
    gzip \
    xz-utils \
    && rm -rf /var/lib/apt/lists/*
RUN curl -L https://github.com/cargo-bins/cargo-binstall/releases/latest/download/cargo-binstall-x86_64-unknown-linux-musl.tgz \
    | tar -xzf - \
    --directory /usr/local/bin
RUN ls /usr/local/bin/
RUN which cargo
RUN cargo-binstall -V
RUN cargo-binstall gnostr --force -y
RUN git init
RUN git config --global user.email "admin@gnostr.org"
RUN git config --global user.name "admin@gnostr.org"
RUN GIT_AUTHOR_DATE="Thu, 01 Jan 1970 00:00:00 +0000" GIT_COMMITTER_DATE="Thu, 01 Jan 1970 00:00:00 +0000" git commit --allow-empty -m 'Initial commit'
ENTRYPOINT ["bash"]
#docker run  -it gnostr:latest -c "git init && gnostr tui --gitdir ."
