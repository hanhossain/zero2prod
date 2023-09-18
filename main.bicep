param webAppName string = uniqueString(resourceGroup().id)
param sku string = 'F1'
param location string = resourceGroup().location
param acrName string = 'acr${uniqueString(resourceGroup().id)}'

var appServicePlanName = toLower('AppServicePlan-${webAppName}')

resource appServicePlan 'Microsoft.Web/serverfarms@2022-09-01' = {
  location: location
  name: appServicePlanName
  properties: {
    reserved: true
  }
  sku: {
    name: sku
  }
  kind: 'linux'
}

resource containerRegistry 'Microsoft.ContainerRegistry/registries@2023-08-01-preview' = {
  location: location
  name: acrName
  sku: {
    name: 'Basic'
  }
  properties: {
    adminUserEnabled: false
  }
}

output acrName string = acrName
output acrLoginServer string = containerRegistry.properties.loginServer
