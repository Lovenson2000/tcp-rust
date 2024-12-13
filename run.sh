#!/bin/bash
cargo build --release
ext=$?
if [[ $ext -ne 0 ]]; then
    exit $ext
fi
sudo setcap cap_net_admin=eip ./target/release/tcp-imple
./target/release/tcp-imple &
pid=$!
sudo ip addr add 192.168.0.1/24 dev tun0
sudo ip link set up dev tun0
trap "kill $pid" INT TERM
wait $pid