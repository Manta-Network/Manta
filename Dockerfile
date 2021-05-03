FROM ubuntu:20.04 as builder
LABEL description="The first stage for building a release manta binary."

ARG PROFILE=release
WORKDIR /src

ENV DEBIAN_FRONTEND noninteractive

COPY . /src

RUN apt update && \
    apt install -y git clang curl libssl-dev

RUN curl https://sh.rustup.rs -sSf | sh -s -- -y && \
	export PATH="$PATH:$HOME/.cargo/bin" && \
	rustup toolchain install nightly && \
	rustup target add wasm32-unknown-unknown --toolchain nightly && \
	rustup default nightly && \
	rustup default stable && \
	cargo build "--$PROFILE"

# ===== SECOND STAGE ======

FROM ubuntu:20.04
LABEL description="The second stage for configuring the image."
ARG PROFILE=release

RUN apt-get update && \
	apt install -y openssl libssl-dev

RUN rm -rf /usr/share/* && \
	useradd -m -u 1000 -U -s /bin/sh -d /manta manta && \
	mkdir -p /manta/.local && \
	chown -R manta:manta /manta/.local

COPY --from=builder /src/target/$PROFILE/manta /usr/local/bin

# checks
RUN ldd /usr/local/bin/manta && \
	/usr/local/bin/manta --version

# Shrinking
RUN rm -rf /usr/lib/python* && \
	rm -rf /src

USER manta
EXPOSE 30333 9933 9944
VOLUME ["/manta"]

ENTRYPOINT ["/usr/local/bin/manta"]
CMD ["/usr/local/bin/manta"]

ENV DEBIAN_FRONTEND teletype
