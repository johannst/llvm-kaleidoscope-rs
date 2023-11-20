FROM ubuntu

RUN apt update && \
    DEBIAN_FRONTEND=noninteractive \
    apt install \
      --yes \
      --no-install-recommends \
      ca-certificates \
      build-essential \
      cargo \
      llvm-15 \
      llvm-15-dev \
      # For polly dependency.
      # https://gitlab.com/taricorp/llvm-sys.rs/-/issues/13
      libclang-common-15-dev \
      zlib1g-dev \
      && \
    rm -rf /var/lib/apt/lists/* && \
    apt-get clean
