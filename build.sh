#!/bin/bash

#Build Flag

FUNCTION=$1
export GOPATH=$HOME/go
export PATH=$PATH:$GOROOT/bin:$GOPATH/bin

export PATH=/usr/local/go/bin:$PATH
export PATH=$HOME/go/bin:$PATH

CreateEnv() {
    sudo apt-get update && sudo apt upgrade -y
    sudo apt-get install make build-essential gcc git jq chrony -y
    wget https://golang.org/dl/go1.18.1.linux-amd64.tar.gz
    sudo tar -C /usr/local -xzf go1.18.1.linux-amd64.tar.gz
    rm -rf go1.18.1.linux-amd64.tar.gz

    export GOROOT=/usr/local/go
    export GOPATH=$HOME/go
    export GO111MODULE=on
    export PATH=$PATH:/usr/local/go/bin:$HOME/go/bin

    ########### install cargo ###########
    sudo apt-get update
    sudo apt-get -y install cargo

    ########### Rustc setup ##########
    sudo apt install -y curl gcc make build-essential
    curl https://sh.rustup.rs -sSf | sh
    source ~/.profile
    source ~/.cargo/env

    rustup default stable
    rustup target add wasm32-unknown-unknown
}

######################### Build and Clean ###############

Build() {

    mkdir out

    echo "================================================="
    echo "Rust Build Start"
    cargo build

    echo "Rust Unit Test"
    cargo test

    cp -r target/debug/*.d  out/
    cp -r target/debug/*.so  out/
    cp -r target/debug/*.rlib out/

    mkdir out/examples
    cp -R target/debug/examples/client out/examples/
    cp -R target/debug/examples/server out/examples/
    
    rm -rf target
}

Clean() {
    rm -rf out
    rm -rf target
}

#################################### End of Function ###################################################
if [[ $FUNCTION == "" ]]; then
    Build
else
    $FUNCTION
fi
