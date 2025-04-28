Rust memory dump tool

### Usage
```
RUST_LOG=debug sudo -E $(which cargo) run -- -p <PID> &> output.txt
```
or
```
sudo $(which cargo) build
RUST_LOG=debug sudo -E ./target/debug/seer -p <PID> &> output.txt
```
