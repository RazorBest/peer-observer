#!/bin/bash

trap 'kill $!' EXIT

while true; do
    echo "Starting dummy"
    ../../target/release/bitcoind_usdt_dummy & echo $! > pidfile

    sleep 10

    kill $(cat pidfile)
    wait $(cat pidfile) 2>/dev/null
done
