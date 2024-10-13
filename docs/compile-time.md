time RUST_LOG="info" cargo run --release benchmark/fib.clang.wasm

| | O0 | O1 | O2 | O3 |
|---|---|---|---|---|
| no-lto | 0.715s | 0.923s | 9.96s | 0.98s |
| lto | 0.78s | 0.99s | 1.2s | 1.15s |
| CR lto | 1.2s | 3.8s | 5.7s | 5.5s |
| CR no-lto | 1.2s | 3.5s | 4.1s | 4.6s |
| R lto | 0.93s | inf? | - | - |
| R no-lto  | 0.81s | inf? | - | - |
| C lto | 1.0s | 2.4s | - | - |