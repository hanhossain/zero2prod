param webAppName string = uniqueString(resourceGroup().id)
param sku string = 'F1'
param location string = resourceGroup().location

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
