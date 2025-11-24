# Setting up Certm and installing dependencies
FROM alpine:latest
RUN apk update && apk add build-base musl-dev openssl-dev openssl-libs-static zlib-dev zlib-static pkgconf && apk add curl && curl https://sh.rustup.rs -sSf | sh -s -- -y && apk add ttyd
ENV PATH="/root/.cargo/bin:/usr/bin:$PATH"
RUN . "$HOME/.cargo/env" && cargo install certm
COPY certs /root/.ca_vault
CMD ["ttyd", "-W", "-p", "8087", "certm"]
