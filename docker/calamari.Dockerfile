FROM ubuntu:20.04 as builder
LABEL description="run calamari binary distribution in docker"
ARG TAG_NAME
ARG BINARY="https://github.com/Manta-Network/Manta/releases/download/${TAG_NAME}/calamari-pc"
ARG CALAMARI_GENESIS="https://raw.githubusercontent.com/Manta-Network/Manta/${TAG_NAME}/genesis/calamari-genesis.json"
ARG KUSAMA_GENESIS="https://raw.githubusercontent.com/paritytech/polkadot/v0.9.9-1/node/service/res/kusama.json"

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
    chmod +x /usr/local/bin/calamari-pc && \
    ldd /usr/local/bin/calamari-pc && \
    /usr/local/bin/calamari-pc --version

EXPOSE 30333 9933 9944 9615
VOLUME ["/calamari"]

ENTRYPOINT ["/usr/local/bin/calamari-pc"]
CMD ["/usr/local/bin/calamari-pc"]

ENV DEBIAN_FRONTEND teletype
