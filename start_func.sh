cargo build
cp ./target/debug/azfunc_sharepoint_uploader .
RUST_LOG=info func start --verbose
