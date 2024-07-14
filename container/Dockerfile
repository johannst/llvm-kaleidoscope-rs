FROM ubuntu

RUN apt update && \
    DEBIAN_FRONTEND=noninteractive \
    apt install \
      --yes \
      --no-install-recommends \
      ca-certificates \
      build-essential \
      cargo \
      llvm-16-dev \
      # For polly dependency.
      # https://gitlab.com/taricorp/llvm-sys.rs/-/issues/13
      libpolly-16-dev \
      libz-dev \
      libzstd-dev \
      && \
    rm -rf /var/lib/apt/lists/* && \
    apt-get clean
