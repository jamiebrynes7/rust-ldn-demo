# Demo for Rust London Meetup

This is a simple SpatialOS project that demonstrates the Rust integration. 

## Prerequisites 

- A SpatialOS account
- Rust v1.34

## Setup & Run the demo

1. Clone https://www.github.com/jamiebrynes7/spatialos-sdk-rs alongside this repository and checkout the `feature/view-impl` branch. 
2. Run `cargo install --path <path_to_spatialos-sdk-rs>/cargo-spatial`.
3. Run `cargo spatial download sdk` to download the C API dependencies.
4. Run `cargo spatial codegen` to generate the code for schema.
5. Run `cargo spatial local launch` to build the workers and launch a local deployment.

