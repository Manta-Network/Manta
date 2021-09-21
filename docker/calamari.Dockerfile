FROM ubuntu:20.04 as builder
LABEL description="parachain node"

ARG PARA_BINARY_URL=https://github.com/Manta-Network/Manta/releases/download/v3.0.4-calamari.kusama/calamari-pc
ARG PARA_BINARY_PATH=/usr/local/bin/parachain
ARG PARA_GENESIS_URL=https://raw.githubusercontent.com/Manta-Network/Manta/manta-pc/genesis/calamari-genesis.json
ARG PARA_GENESIS_PATH=/usr/share/parachain.json
ARG RELAY_GENESIS_URL=https://raw.githubusercontent.com/paritytech/polkadot/v0.9.9-1/node/service/res/kusama.json
ARG RELAY_GENESIS_PATH=/usr/share/relaychain.json

ARG SUBSTRATE_BASE_PATH=/var/lib/substrate
ARG SUBSTRATE_PORT=30333
ARG SUBSTRATE_RPC_PORT=9933
ARG SUBSTRATE_RPC_CORS=all
ARG SUBSTRATE_WS_PORT=9944
ARG SUBSTRATE_WS_MAX_CONNECTIONS=100
ARG SUBSTRATE_PARACHAIN_ID=2084

# install deps
RUN apt-get update
RUN apt-get upgrade -y
ARG DEBIAN_FRONTEND=noninteractive
RUN apt-get install -yq openssl
RUN apt-get install -yq libssl-dev

RUN mkdir -p /usr/local/bin
RUN mkdir -p /usr/share
RUN mkdir -p ${SUBSTRATE_BASE_PATH}

ADD ${PARA_BINARY_URL} ${PARA_BINARY_PATH}
RUN chmod +x ${PARA_BINARY_PATH}
RUN ldd ${PARA_BINARY_PATH}
RUN ${PARA_BINARY_PATH} --version

ADD ${PARA_GENESIS_URL} ${PARA_GENESIS_PATH}
ADD ${RELAY_GENESIS_URL} ${RELAY_GENESIS_PATH}

EXPOSE ${SUBSTRATE_PORT}
EXPOSE ${SUBSTRATE_RPC_PORT}
EXPOSE ${SUBSTRATE_WS_PORT}

ENTRYPOINT ["/usr/local/bin/calamari"]
CMD [ \
    "--chain", "${PARA_GENESIS_PATH}", \
    "--base-path", "${SUBSTRATE_BASE_PATH}", \
    "--parachain-id", "${SUBSTRATE_PARACHAIN_ID}", \
    "--port", "${SUBSTRATE_PORT}", \
    "--rpc-port", "${SUBSTRATE_RPC_PORT}", \
    "--ws-port", "${SUBSTRATE_WS_PORT}", \
    "--rpc-cors", "${SUBSTRATE_RPC_CORS}", \
    "--ws-max-connections", "${SUBSTRATE_WS_MAX_CONNECTIONS}", \
    "--bootnodes", \
    "/dns/crispy.calamari.systems/tcp/30333/p2p/12D3KooWNE4LBfkYB2B7D4r9vL54YMMGsfAsXdkhWfBw8VHJSEQc", \
    "/dns/crunchy.calamari.systems/tcp/30333/p2p/12D3KooWL3ELxcoMGA6han3wPQoym5DKbYHqkWkCuqyjaCXpyJTt", \
    "/dns/deepfried.calamari.systems/tcp/30333/p2p/12D3KooWBdto53HnArmLdtf2RXzNWti7hD5mML7DWGZPD8q4cywv", \
    "/dns/grilled.calamari.systems/tcp/30333/p2p/12D3KooWDTrkWDvkKnGxFAh9LRLW3eh2n4X2AHs85BwK6JhA89Uj", \
    "/dns/hotdog.calamari.systems/tcp/30333/p2p/12D3KooWMHdpUCCS9j8hvNLTV8PeqJ16KaVEjb5PVdYgAQUFUcCG", \
    "/dns/tasty.calamari.systems/tcp/30333/p2p/12D3KooWGs2hfnRQ3Y2eAoUyWKUL3g7Jmcsf8FpyhVYeNpXeBMSu", \
    "/dns/tender.calamari.systems/tcp/30333/p2p/12D3KooWNXZeUSEKRPsp1yiDH99qSVawQSWHqG4umPjgHsn1joci", \
    "--", \
    "--chain", "${RELAY_GENESIS_PATH}" \
]
