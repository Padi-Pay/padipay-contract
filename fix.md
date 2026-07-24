Description
The Stellar network imposes strict limits on the size of deployed WebAssembly contracts to maintain network performance. Unoptimized Rust contracts can easily exceed these limits or incur high deployment costs. Configuring the release profile drastically minimizes the deployment footprint.

Requirements & Context
Architectural Goal: Optimize the compiled .wasm artifact for the Soroban environment.

Implementation Expectations: Audit and tighten the Cargo.toml release profile. Introduce configurations such as lto = true, opt-level = "z", codegen-units = 1, and panic = "abort". Ensure the stellar contract optimize pipeline is utilized in the build script.

Acceptance Criteria

The compiled release.wasm file size is measurably reduced.

Performance characteristics of the optimized build remain acceptable.

Tests continue to pass on the optimized binary.
Out of Scope
Removing critical features purely for bundle size reductions.
Suggested Execution
git checkout -b ci/bundle-optimization
Suggested Commit Message
ci: optimize cargo profile for minimal wasm bundle size
Testing Notes
Compare WASM size before and after the Cargo.toml changes.
Ensure the build process succeeds locally and in CI.
References
Soroban Contract Optimization best practices.
Cargo.toml profile documentation.

Definition of Done

Ready for review.