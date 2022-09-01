FROM ubuntu:20.04
LABEL description="calamari parachain node"

ARG PARA_BINARY_REF

ARG PARA_GENESIS_REF=manta
ARG PARA_BINARY_URL=https://github.com/Manta-Network/Manta/releases/download/$PARA_BINARY_REF/manta
ARG PARA_BINARY_PATH=/usr/local/bin/manta

ARG PARA_GENESIS_URL=https://raw.githubusercontent.com/Manta-Network/Manta/$PARA_GENESIS_REF/genesis/calamari-genesis.json
ARG PARA_GENESIS_PATH=/usr/share/calamari.json

ARG RELAY_GENESIS_URL=https://raw.githubusercontent.com/paritytech/polkadot/master/node/service/chain-specs/kusama.json
ARG RELAY_GENESIS_PATH=/usr/share/kusama.json

# Install deps
RUN apt-get update
RUN apt-get upgrade -y
ARG DEBIAN_FRONTEND=noninteractive
RUN apt-get install -yq openssl libssl-dev

RUN mkdir -p /usr/local/bin
RUN mkdir -p /usr/share

# Dowload latest calamari binary
ADD $PARA_BINARY_URL $PARA_BINARY_PATH
RUN chmod +x $PARA_BINARY_PATH
RUN ldd $PARA_BINARY_PATH
RUN $PARA_BINARY_PATH --version

# Get calamari and kusama genesis file
ADD $PARA_GENESIS_URL $PARA_GENESIS_PATH
ADD $RELAY_GENESIS_URL $RELAY_GENESIS_PATH

# Expose 5 ports by default
EXPOSE 30333 30334 9933 9944 9615 9945

ENTRYPOINT [\
  "/usr/local/bin/manta",\
  "--chain",\
  "/usr/share/calamari.json"\
]
