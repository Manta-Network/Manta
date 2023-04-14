FROM ubuntu:20.04
LABEL description="manta parachain node"

ARG PARA_BINARY_REF

ARG PARA_GENESIS_REF=manta
ARG PARA_BINARY_URL=https://github.com/Manta-Network/Manta/releases/download/$PARA_BINARY_REF/manta
ARG PARA_BINARY_PATH=/usr/local/bin/manta

ARG PARA_GENESIS_URL=https://raw.githubusercontent.com/Manta-Network/Manta/$PARA_GENESIS_REF/genesis/manta-genesis.json
ARG PARA_GENESIS_PATH=/usr/share/manta.json

ARG RELAY_GENESIS_URL=https://raw.githubusercontent.com/paritytech/polkadot/master/node/service/chain-specs/polkadot.json
ARG RELAY_GENESIS_PATH=/usr/share/polkadot.json

# install deps
RUN apt-get update
RUN apt-get upgrade -y
ARG DEBIAN_FRONTEND=noninteractive
RUN apt-get install -yq openssl
RUN apt-get install -yq libssl-dev

RUN mkdir -p /usr/local/bin
RUN mkdir -p /usr/share

ADD $PARA_BINARY_URL $PARA_BINARY_PATH
RUN chmod +x $PARA_BINARY_PATH
RUN ldd $PARA_BINARY_PATH
RUN $PARA_BINARY_PATH --version

# Get manta and kusama genesis file
ADD $PARA_GENESIS_URL $PARA_GENESIS_PATH
ADD $RELAY_GENESIS_URL $RELAY_GENESIS_PATH

# Expose 5 ports by default
EXPOSE 30333 30334 9933 9944 9615 9945
ENTRYPOINT [\
  "/usr/local/bin/manta",\
  "--chain",\
  "/usr/share/manta.json"\
]
