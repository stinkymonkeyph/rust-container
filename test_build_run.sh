#!/bin/bash
cargo build --release
sudo ./target/release/rust-container run /bin/bash
