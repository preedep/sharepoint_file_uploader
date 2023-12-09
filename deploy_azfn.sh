./build_linux.sh

cp ./target/x86_64-unknown-linux-musl/release/azfunc_sharepoint_uploader .

# Pack zip files
rm -rf deployment.zip
zip -r deployment.zip azfunc_sharepoint_uploader host.json HttpTriggerCopyBlob2SPO/

# Deployment
