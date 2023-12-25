#!/bin/bash

# build the binary
cargo b --release

# if build compilation fails then dont proceed and exit the script
if [[ $? -ne 0 ]]; then
    exit 
fi

# set Binary capabilities to perform Network operations
sudo setcap CAP_NET_ADMIN=eip target/release/tcp-rust

# run the binary
target/release/tcp-rust & 
pid=$!

# setup a TUN interface
sudo ip addr add 192.168.0.1/24 dev tun0
sudo ip link set up dev tun0

# gracefully shutdown the running Binary on Ctrl-C, instead of sending it to bg
trap "kill $pid" INT TERM
wait $pid

# new line 
echo 