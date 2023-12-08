# Share Point File Uploader

POC (Prove Of Concept) for Upload file from Azure Blob Storage to Share Point Online

# Prerequisite:
- Share Point Online
- Azure Portal

# Azure portal
- Create New Application Account in App Registerations
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