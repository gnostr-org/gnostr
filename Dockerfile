FROM rust:latest as base
LABEL org.opencontainers.image.source="https://github.com/gnostr-org/gnostr"
LABEL org.opencontainers.image.description="gnostr-docker"
RUN touch updated
RUN echo $(date +%s) > updated
RUN apt-get update
RUN apt-get install -y bash cmake git make tcl-dev
WORKDIR /tmp
RUN git clone --recurse-submodules -j4 --branch  1709070216/8906a5ae7/2cfe0e22f --depth 10 https://github.com/gnostr-org/gnostr.git
WORKDIR /tmp/gnostr
RUN make detect
RUN make gnostr-am
FROM base as gnostr
RUN cmake .
RUN make gnostr
ENV SUDO=sudo
RUN make        install
RUN make gnostr-install
RUN cargo install --path bins --force
RUN install ./serve /usr/local/bin || true
ENV PATH=$PATH:/usr/bin/systemctl
RUN ps -p 1 -o comm=
EXPOSE 80 6102 8080 ${PORT}
VOLUME /src
FROM gnostr as gnostr-docker

