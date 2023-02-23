# CCA Club Hub Backend

This the backend for the CCA Club Hub written in Rust.  
If you are new to Rust, consider reading [The Rust Programming Language](https://doc.rust-lang.org/book/) to gain a basic understanding of how Rust works.

# Installation
1. install git
    - https://git-scm.com/book/en/v2/Getting-Started-Installing-Git
2. install cargo
    - https://doc.rust-lang.org/cargo/getting-started/installation.html
3. clone this repository
    - in a terminal:
        ```
        git clone https://github.com/Coding-Outreach/cca-club-hub-rust-backend
        cd cca-club-hub-rust-backend
        ```

# Setup (**required**)
- copy the `.env.sample` file into a `.env` file
- fill out the fields in the `.env` file

# Deployment
- `cargo run`
    - deploys the backend in debug mode
    - safety checks, less optimizations, meant for development
- `cargo run --release`
    - deploys the backend in release mode
    - faster, meant for production