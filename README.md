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
test_yaml_path: --String, path to the test scene, default to "map_file/maze-32-32-2-scen-even/maze-32-32-2-even-1.scen"
test_map_path: --String, path to the mao, default to "map_file/maze-32-32-2-scen-even/maze-32-32-2.map"
num_agents: --Uint, number of agents, default to 10
agents_dist: --Optional<List of Uint>, Specify scene buckets 
seed: --Uint, random seed, default to 0
sub_optimal: 
  - --Float, high level suboptimal constant, default to 1.0
  - --Float, low level suboptimal constant, default to 1.0
solver: --List of solver, choose in CBS, HBCBS. LBCBS, BCBS, ECBS
```

Run the benchmark

```
cargo run --release -- --config ./conf/$(test script)
```