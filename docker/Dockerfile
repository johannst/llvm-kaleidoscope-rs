FROM ubuntu

RUN apt update && \
    DEBIAN_FRONTEND=noninteractive \
    apt install \
      --yes \
      --no-install-recommends \
      ca-certificates \
      build-essential \
      cargo \
      llvm-14 \
      llvm-14-dev \
      # https://gitlab.com/taricorp/llvm-sys.rs/-/issues/13
      libclang-common-14-dev \
      zlib1g-dev \
      && \
    rm -rf /var/lib/apt/lists/* && \
    apt-get clean
