This project is actively being developed to implement both optimal and suboptimal variants of the Conflict-Based Search (CBS) algorithm for Multi-Agent Path Finding (MAPF) using Rust.

## Environment
Install the [Rust toolchain](https://www.rust-lang.org/tools/install)

## Usage
Build the project

```
cargo build --release
```

Config test scripts and save as `yaml` file under `config` filefolder, an example is showed in `conf/test.yaml`

```
yaml-path               | Path to the YAML scenario file
map-file                | Path to the map file
num-agents              | Number of agents
agents-dist             | Distribution of agents buckets
seed                    | Seed for the random number generator
low-level-sub-optimal   | Suboptimal limit for low-level operations
high-level-sub-optimal  | Suboptimal limit for high-level operations
solver                  | Solver to use
```

Run the solver

```
cargo run --release
```