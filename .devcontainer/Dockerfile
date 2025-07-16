FROM mcr.microsoft.com/devcontainers/base:ubuntu

WORKDIR /workspaces/hot-or-not-web-leptos-ssr/

RUN sudo apt update
RUN sudo apt full-upgrade --yes


# Enable the add-apt-repository command
RUN sudo apt install --yes software-properties-common

# Add the main repository for mold
RUN sudo add-apt-repository main universe --yes

# Install mold
RUN sudo apt install --yes mold pkg-config rustup

# Install nushell
RUN curl -fsSL https://apt.fury.io/nushell/gpg.key | sudo gpg --dearmor -o /etc/apt/trusted.gpg.d/fury-nushell.gpg
RUN echo "deb https://apt.fury.io/nushell/ /" | sudo tee /etc/apt/sources.list.d/fury.list
RUN sudo apt update
RUN sudo apt install nushell

SHELL ["/usr/bin/nu", "-c"]

RUN open rust-toolchain.toml | get toolchain.channel | rustup install $in

# RUN rustc --version

# RUN curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash

# ENV PATH="~/.cargo/bin:${PATH}"