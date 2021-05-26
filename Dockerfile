FROM ubuntu:20.04 as builder
LABEL description="run manta binary distribution in docker"
ARG BINARY="https://github.com/Manta-Network/Manta/releases/download/v3.0.0-alpha.3/manta"

ENV DEBIAN_FRONTEND noninteractive

WORKDIR /manta-bin

ADD "$BINARY" /manta-bin/manta 

RUN apt-get update && \
	apt install -y openssl libssl-dev

# shrink size
RUN rm -rf /usr/share/*

# make executable and check	
RUN cp ./manta /usr/local/bin/manta && \
    chmod 777 /usr/local/bin/manta && \
    ldd /usr/local/bin/manta && \
    /usr/local/bin/manta --version

EXPOSE 30333 9933 9944 9615
VOLUME ["/manta"]

ENTRYPOINT ["/usr/local/bin/manta"]
CMD ["/usr/local/bin/manta"]

ENV DEBIAN_FRONTEND teletype

