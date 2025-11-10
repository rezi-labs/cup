// Azure Bicep template with version tracking
@description('Azure resources deployment template')

// Parameters with version tracking
param containerVersion string = '0.15.0' // [cup] GitHub rezi-labs/rezi-web
param nodeImageVersion string = '0.15.0' // [cup] rezi-labs/rezi-web
param nginxVersion string = '0.15.0' // [cup] rezi-labs/rezi-web

// Variables using different syntax patterns
var appVersion = '0.15.0' // [cup] rezi-labs/rezi-web
var dbVersion := '0.15.0' // [cup] rezi-labs/rezi-web
var redisVersion: '0.15.0' # [cup] rezi-labs/rezi-web

// Resource definitions
resource containerRegistry 'Microsoft.ContainerRegistry/registries@2023-07-01' = {
  name: 'myregistry'
  location: resourceGroup().location
  properties: {
    adminUserEnabled: true
  }
}

// Container instance with fallback version tracking
resource containerInstance 'Microsoft.ContainerInstance/containerGroups@2023-05-01' = {
  name: 'mycontainer'
  location: resourceGroup().location
  properties: {
    containers: [
      {
        name: 'app'
        properties: {
          image: 'node:${nodeImageVersion}' // Version tracked above
          ports: [
            {
              port: 80
            }
          ]
          resources: {
            requests: {
              cpu: 1
              memoryInGB: 2
            }
          }
        }
      }
    ]
    osType: 'Linux'
    restartPolicy: 'Always'
  }
}

// Output with version info
output containerVersion string = containerVersion
output deploymentVersion string = '0.15.0' // [cup] rezi-labs/rezi-web