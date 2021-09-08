FROM ubuntu:20.04 as builder
LABEL description="run calamari binary distribution in docker"
ARG BINARY="https://github.com/Manta-Network/Manta/releases/download/v3.0.1-6147698/calamari-pc"
ARG CALAMARI_GENESIS="https://raw.githubusercontent.com/Manta-Network/Manta/6147698/genesis/calamari-genesis.json"
ARG KUSAMA_GENESIS="https://raw.githubusercontent.com/paritytech/polkadot/f9d5ad2/node/service/res/kusama.json"

ENV DEBIAN_FRONTEND noninteractive

WORKDIR /calamari-bin

ADD "$BINARY" /calamari-bin/calamari-pc 
ADD "$CALAMARI_GENESIS" /calamari-bin/calamari-genesis.json
ADD "$KUSAMA_GENESIS" /calamari-bin/kusama.json

RUN apt-get update && \
	apt install -y openssl libssl-dev

# shrink size
RUN rm -rf /usr/share/*

# make executable and check	
RUN cp ./calamari-pc /usr/local/bin/calamari-pc && \
    chmod 777 /usr/local/bin/calamari-pc && \
    ldd /usr/local/bin/calamari-pc && \
    /usr/local/bin/calamari-pc --version

EXPOSE 30333 9933 9944 9615
VOLUME ["/calamari"]

ENTRYPOINT ["/usr/local/bin/calamari-pc"]
CMD ["/usr/local/bin/calamari-pc"]

ENV DEBIAN_FRONTEND teletype