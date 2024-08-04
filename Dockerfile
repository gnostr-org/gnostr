FROM rust:latest as base
LABEL org.opencontainers.image.source="https://github.com/gnostr-org/gnostr"
LABEL org.opencontainers.image.description="gnostr-docker"
RUN touch updated
RUN echo $(date +%s) > updated
RUN apt-get update && apt-get install autoconf bash build-essential cmake curl git jq libexpat1-dev libcurl4-openssl-dev libssl-dev libtool lsb-release make nodejs npm pkg-config python3 python-is-python3 sudo tcl-dev zlib1g-dev -y

RUN git clone --branch master --depth 1 https://github.com/gnostr-org/gnostr.git
WORKDIR /tmp
RUN git clone --recurse-submodules -j4 --branch master --depth 1 https://github.com/gnostr-org/gnostr.git
WORKDIR /tmp/gnostr
RUN make detect
RUN make gnostr-am
FROM base as gnostr
#RUN cmake .
RUN make gnostr
ENV SUDO=sudo
RUN make gnostr-install
RUN cargo install cargo-binstall --force
RUN cargo-binstall gnostr-bins --force
RUN cargo-binstall gnostrd --force
RUN cargo-binstall gnostr-cli --force
RUN cargo-binstall gnostr-gui --force
RUN cargo-binstall gnostr-lookup --force
RUN cargo-binstall gnostr-tui --force
RUN cargo-binstall nips --force
ENV PATH=$PATH:/usr/bin/systemctl
RUN ps -p 1 -o comm=
EXPOSE 80 6102 8080 ${PORT}
VOLUME /src
FROM gnostr as gnostr-docker
