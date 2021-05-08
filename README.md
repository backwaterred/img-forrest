# 🌲 Image Forest

Welcome to Image Forest, a great place to cache your images to keep them safe during the winter. Any images which are part of the database when a user logs off will be squirreled away for next season.

For a detailed overview of the project, start the server and navigate to http://localhost:8080/

## Install/Test/Run instructions

This project is written in Rust, and tested with Rust 1.45 on Linux. Although it has been designed to be platform independent, it has not been tested on other platforms.

### Install

1. Install Rust by following the directions at https://www.rust-lang.org/tools/install
   - `cargo --version` and `rustc --version` should both execute cleanly.
   
### Test & Run

1. To build the project and run the unit tests: `cargo test`
   - The project should build and run. All tests should pass.
2. To run the server: `cargo run`
   - Test by navigating to http://localhost:8080/
   - Leave the server running, and query any of the endpoints with [Postman](https://www.postman.com/downloads/), curl, or your favourite tool.
