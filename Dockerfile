FROM rust:latest AS build
EXPOSE 4000
WORKDIR /app
RUN apt-get update && apt-get install -y \
    curl \
    gzip \
    sudo \
    tar \
    xz-utils \
    && rm -rf /var/lib/apt/lists/*
RUN curl -L https://github.com/cargo-bins/cargo-binstall/releases/latest/download/cargo-binstall-x86_64-unknown-linux-musl.tgz \
    | tar -xzf - \
    --directory /usr/local/bin
RUN cargo-binstall -V
RUN cargo-binstall cross -y
RUN cargo-binstall gnostr --force -y
RUN git init
RUN git config --global user.email "admin@gnostr.org"
RUN git config --global user.name "admin@gnostr.org"
RUN GIT_AUTHOR_DATE="Thu, 01 Jan 1970 00:00:00 +0000" GIT_COMMITTER_DATE="Thu, 01 Jan 1970 00:00:00 +0000" git commit --allow-empty -m 'gnostr repo'
ENTRYPOINT ["bash"]
