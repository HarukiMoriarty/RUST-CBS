This project is actively being developed to implement both optimal and suboptimal variants of the Conflict-Based Search (CBS) algorithm for Multi-Agent Path Finding (MAPF).

## Environment
Install the [Rust toolchain](https://www.rust-lang.org/tools/install)

## Usage
Build the project

```
cargo build
```

Config test scripts and save as `yaml` file under `config` filefolder

```
test_yaml_path: -- Path to the test scene, default to "map_file/maze-32-32-2-scen-even/maze-32-32-2-even-1.scen"
test_map_path: -- path to the mao, default to "map_file/maze-32-32-2-scen-even/maze-32-32-2.map"
num_agents: -- Number of agents, default to 10
agents_dist: -- <Optional> Specify scene buckets 
seed: -- Random seed, default to 0
sub_optimal: 
  - -- High level suboptimal constant, default to 1.0
  - -- Low level suboptimal constant, default to 1.0
solver: -- Solver, choose in CBS, HBCBS. LBCBS, BCBS, ECBS
```

Run the benchmark

```
cargo run --release -- --config ./conf/$(test script)
```