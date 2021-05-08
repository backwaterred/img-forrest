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
2. To execute the endpoint tests, use [Postman's test runner](https://learning.postman.com/docs/running-collections/intro-to-collection-runs/#running-your-collections).
   - The collection is shared at the link https://www.postman.com/collections/06b88b64a760e3f9ab36
   - Follow the guidance at https://learning.postman.com/docs/getting-started/importing-and-exporting-data/#importing-data-into-postman to import a collection.
   - Start the server from the command line with `cargo run`. Then, using Postman's test runner, execute the tests in the collection. All tests should pass.
   - Exit the server with `Ctrl-c`.
3. To run the server execute `cargo run`
   - Test by navigating to http://localhost:8080/
   - With the server running, query any of the endpoints from [the documentation](http://localhost:8080/).
   - Exit the server with `Ctrl-c`.
