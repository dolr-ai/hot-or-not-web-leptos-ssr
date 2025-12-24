FROM mcr.microsoft.com/devcontainers/base:ubuntu

RUN sudo apt update
RUN sudo apt full-upgrade --yes

# Enable the add-apt-repository command
RUN sudo apt install --yes software-properties-common

# Add the main repository for mold
RUN sudo add-apt-repository main --yes

# Install build dependencies
RUN sudo apt install --yes mold pkg-config musl-tools lld

# Install Node.js 20
RUN curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash - && \
    sudo apt install --yes nodejs

# Install pnpm
RUN sudo npm install -g pnpm@8