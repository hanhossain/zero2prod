param webAppName string = uniqueString(resourceGroup().id)
param location string = resourceGroup().location

var acrName = 'acr${uniqueString(resourceGroup().id)}'
var appServicePlanName = toLower('AppServicePlan-${webAppName}')
var websiteName = toLower('wapp-${webAppName}')

resource appServicePlan 'Microsoft.Web/serverfarms@2022-09-01' = {
  location: location
  name: appServicePlanName
  properties: {
    reserved: true
  }
  sku: {
    name: 'F1'
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

resource appService 'Microsoft.Web/sites@2022-09-01' = {
  location: location
  name: websiteName
  identity: {
    type: 'SystemAssigned'
  }
  properties: {
    serverFarmId: appServicePlan.id
    siteConfig: {
      acrUseManagedIdentityCreds: true
      linuxFxVersion: 'DOCKER|${containerRegistry.properties.loginServer}/zero2prod:latest'
    }
  }
}

resource acrPullRoleDefinition 'Microsoft.Authorization/roleDefinitions@2022-05-01-preview' existing = {
  name: '7f951dda-4ed3-4680-a7ca-43fe172d538d'
}

resource appServicePullFromContainerRegistryRole 'Microsoft.Authorization/roleAssignments@2022-04-01' = {
  name: guid(resourceGroup().id, appService.id, acrPullRoleDefinition.id)
  scope: containerRegistry
  properties: {
    principalId: appService.identity.principalId
    roleDefinitionId: acrPullRoleDefinition.id
  }
}

output acrName string = acrName
output acrLoginServer string = containerRegistry.properties.loginServer
