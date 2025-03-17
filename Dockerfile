FROM debian:bookworm-slim AS builder

WORKDIR /wanco

COPY . /wanco

# install dependencies
RUN \
    --mount=type=cache,target=/var/cache/apt \    
    apt-get update && apt-get install -y \
    ca-certificates lsb-release wget software-properties-common gnupg \
    git \
    tar \
    build-essential \
    cmake \
    libprotobuf-dev \
    protobuf-compiler \
    libunwind-dev \
    libelf-dev \
    libzstd-dev \
    libpolly-17-dev

# install llvm
RUN --mount=type=cache,target=/usr/lib/llvm-17 \
    wget -O llvm.sh https://apt.llvm.org/llvm.sh && \
    chmod +x llvm.sh && \
    ./llvm.sh 17
ENV LLVM_SYS_170_PREFIX=/usr/lib/llvm-17

# install Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

# Build and install wanco
RUN mkdir build && cd build && cmake -DCMAKE_BUILD_TYPE=Release .. && make && make install

#ENTRYPOINT ["wanco"]