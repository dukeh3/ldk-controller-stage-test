# Jenkins inbound agent with Rust toolchain and Debian packaging tools.
# Base: jenkins/inbound-agent:jdk17 (Debian bookworm).
#
# Build:
#   docker build -t jenkins-agent-rust .
#
# Intended labels: docker debian rust

FROM jenkins/inbound-agent:jdk17

USER root

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
       build-essential \
       pkg-config \
       libssl-dev \
       debhelper \
       devscripts \
       dpkg-dev \
       fakeroot \
       lintian \
       reprepro \
       openssh-client \
       git \
       curl \
       ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Allow jenkins user to access Docker socket (GID 107 on user-service-host)
RUN groupadd -g 107 docker && usermod -aG docker jenkins

# Install docker-compose v2 standalone
RUN curl -fsSL "https://github.com/docker/compose/releases/latest/download/docker-compose-linux-x86_64" \
    -o /usr/local/bin/docker-compose && chmod +x /usr/local/bin/docker-compose

USER jenkins
ENV CARGO_HOME=/home/jenkins/.cargo
ENV RUSTUP_HOME=/home/jenkins/.rustup
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
    | sh -s -- -y --default-toolchain stable --profile minimal
ENV PATH="/home/jenkins/.cargo/bin:${PATH}"
