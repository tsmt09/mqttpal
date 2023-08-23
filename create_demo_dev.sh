#!/bin/bash

# Create a demo user
cargo run -- create-user super dev
cargo run -- create-client test "mqtt://192.168.21.4:1883?client_id=mqttweb"