FROM ubuntu:20.04
LABEL description="calamari parachain node"

ARG PARA_BINARY_REF

ARG PARA_GENESIS_REF=manta
ARG PARA_BINARY_URL=https://github.com/Manta-Network/Manta/releases/download/$PARA_BINARY_REF/manta
ARG PARA_BINARY_PATH=/usr/local/bin/manta

ARG PARA_GENESIS_URL=https://raw.githubusercontent.com/Manta-Network/Manta/$PARA_GENESIS_REF/genesis/calamari-genesis.json
ARG PARA_GENESIS_PATH=/usr/share/calamari.json

ARG RELAY_GENESIS_URL=https://raw.githubusercontent.com/paritytech/polkadot/master/node/service/res/kusama.json
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
EXPOSE 30333 30334 9933 9944 9615

ENTRYPOINT [\
  "/usr/local/bin/manta",\
  "--chain",\
  "/usr/share/calamari.json",\
  "--bootnodes",\
  "/dns/crispy.calamari.systems/tcp/30333/p2p/12D3KooWNE4LBfkYB2B7D4r9vL54YMMGsfAsXdkhWfBw8VHJSEQc", \
  "/dns/crunchy.calamari.systems/tcp/30333/p2p/12D3KooWL3ELxcoMGA6han3wPQoym5DKbYHqkWkCuqyjaCXpyJTt", \
  "/dns/hotdog.calamari.systems/tcp/30333/p2p/12D3KooWMHdpUCCS9j8hvNLTV8PeqJ16KaVEjb5PVdYgAQUFUcCG", \
  "/dns/tasty.calamari.systems/tcp/30333/p2p/12D3KooWGs2hfnRQ3Y2eAoUyWKUL3g7Jmcsf8FpyhVYeNpXeBMSu", \
  "/dns/tender.calamari.systems/tcp/30333/p2p/12D3KooWNXZeUSEKRPsp1yiDH99qSVawQSWHqG4umPjgHsn1joci" \
]
