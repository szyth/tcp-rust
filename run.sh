#!/bin/bash
cargo b --release
sudo setcap CAP_NET_ADMIN=eip target/release/tcp-rust
target/release/tcp-rust & 
pid=$!
sudo ip addr add 192.168.0.1/24 dev tun0
sudo ip link set up dev tun0
trap "kill $pid" INT TERM
wait $pid
echo 