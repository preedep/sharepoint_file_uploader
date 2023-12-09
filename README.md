# Share Point File Uploader

POC (Proof Of Concept) for Upload file from Azure Blob Storage to Share Point Online

**_My POC is using Rust_**, but you can use any language you want, as long as you can get the token from Share Point Online
In this project have 2 main application interfaces
- CLI (Command Line Interface)
- Rest API (For Azure Function)


# Prerequisite:
- Share Point Online
- Azure Portal

# Azure portal
- Create New Application Account in App Registrations
- Create New Secret Key in your new Application Account

# Share Point Online
- If you never access Share Point Online, via Rest API and uses Token Authentication, you can use m365 cli do like this
```bash
npm i -g @pnp/cli-microsoft365

m365 spo tenant settings set --DisableCustomAppAuthentication false
```


**for more configuration, please refer to this [link](https://www.syntera.ch/blog/2022/10/10/copy-files-from-sharepoint-to-blob-storage-using-azure-data-factory/)** (Configuration for Copy file from Share Point to Azure Blob Storage via Azure Data Factory)



```
AZURE_TENANT_ID=xxxxxx \
AZURE_CLIENT_ID=xxxx \
AZURE_CLIENT_SECRET=xxxxxx \
RUST_LOG=debug ./target/debug/sharepoint_uploader --storage-account "xx" \
  --container-name "xx" \
  --blob-name "xxx.txt" \
  --spo-domain "--domain" \
  --spo-site "--site" \
  --spo-path "--path
```
AZURE_* Get from Azure App Registration in Azure Portal

SHARE_POINT_DOMAIN Get from Share Point Online


# Azure Function 
For test locally, you can use this command
```
cargo build
cp ./target/debug/azfunc_sharepoint_uploader .
RUST_LOG=debug func start --verbose
```
Curl Test
```
curl -v -X POST http://localhost:7071/api/HttpTriggerCopyBlob2SPO -H 'Content-Type: application/json' \
    -d '{
          "tenant_id": "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx",
          "client_id": "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx",
          "client_secret": "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx",
          "share_point_domain": "1234",
          "share_point_site": "XX",
          "share_point_path": "/sites/xxx/yyyyyy",
          "account": "xxx",
          "container": "xxx",
          "blob_name": "xxx.txt"
         }'
``` 
Build for Azure Function (Linux) (for my case I use macOS)
```
brew tap SergioBenitez/osxct
brew install FiloSottile/musl-cross/musl-cross
```

```
rustup target add x86_64-unknown-linux-musl
TARGET_CC=x86_64-linux-musl-gcc \
RUSTFLAGS="-C linker=x86_64-linux-musl-gcc" \
cargo build --release --target=x86_64-unknown-linux-musl
```

Package for Azure Function
```
./build_linux.sh

cp ./target/x86_64-unknown-linux-musl/release/azfunc_sharepoint_uploader .

# Pack zip files
rm -rf deployment.zip
zip -r deployment.zip azfunc_sharepoint_uploader host.json HttpTriggerCopyBlob2SPO/
```

Azure CLI to deploy zip file to Azure Function
```
az functionapp deployment source config-zip -g <resource_group> -n <app_name> --src deployment.zip
```