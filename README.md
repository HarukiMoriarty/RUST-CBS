This project is actively being developed to implement both optimal and suboptimal variants of the Conflict-Based Search (CBS) algorithm for Multi-Agent Path Finding (MAPF) using Rust.

## Environment
Install the [Rust toolchain](https://www.rust-lang.org/tools/install)

## Usage
Build the project

```
cargo build --release
```

Config test scripts and save as `yaml` file under `config` filefolder, an example is showed in `conf/test.yaml`


| Parameter                  | Description                                      |
|---------------------------|--------------------------------------------------|
| `--yaml-path`             | Path to the YAML scenario file                   |
| `--map-path`              | Path to the map file                             |
| `--output-path`           | Output path for runtime statistics (CSV, etc.)   |
| `--solution-path`         | Output path for LACAM-style formatted solution   |
| `--num-agents`            | Number of agents                                 |
| `--agents-dist`           | Distribution of agents buckets                   |
| `--seed`                  | Random number generator seed                     |
| `--low-level-sub-optimal` | Suboptimal limit for low-level operations        |
| `--high-level-sub-optimal`| Suboptimal limit for high-level operations       |
| `--solver`                | Solver to use (`cbs`, `ecbs`, `bcbs`, etc.)      |
| `--debug-yaml`            | Enable debugging with hardcoded `debug.yaml`     |
| `--op-prioritize-conflicts` | Optimization: Prioritize conflicts             |
| `--op-bypass-conflicts`   | Optimization: Bypass conflicts                   |
| `--op-target-reasoning`   | Optimization: Target reasoning                   |


Run the solver

```
cargo run --release -- $(paras)
```