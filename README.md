#YeoHeng

This repository contains the main server used to run the YeoHeng project.

#How to run
###Basic 
1. Simply run the project using `cargo run`
###Running with autoreload
1. Install systemfd by running `cargo install systemfd cargo-watch`
2. Once systemfd is installed run the server using `systemfd --no-pid -s http::3000 -- cargo watch -x run`