cargo build
cp ./target/debug/azfunc_sharepoint_uploader .
RUST_LOG=debug func start --verbose