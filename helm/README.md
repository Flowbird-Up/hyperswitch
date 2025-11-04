# Helm Chart pour hyperswitch

Ce dossier contient la configuration Helm pour déployer hyperswitch sur Kubernetes.

## Structure

- `charts/` - Définition du chart Helm
- `values/` - Fichiers de configuration pour différents environnements

## Environnements Disponibles

### Staging
```bash
# Développement
helm install hyperswitch ./helm/charts -f ./helm/values/staging-hyperswitch-dev-values.yaml

# Intégration  
helm install hyperswitch ./helm/charts -f ./helm/values/staging-hyperswitch-int-values.yaml
```

### Production
```bash
# Production EU
helm install hyperswitch ./helm/charts -f ./helm/values/prod-eu-hyperswitch-prod-values.yaml

# Production US
helm install hyperswitch ./helm/charts -f ./helm/values/prod-us-hyperswitch-prod-values.yaml
```

### Local Development
```bash
# Minikube local
helm install hyperswitch ./helm/charts -f ./helm/values/minikube-hyperswitch-local-values.yaml
```

## Configuration GP

Cette configuration est préparée pour le déploiement sur la plateforme GP (Global Platform).

## Configuration

Modifiez les fichiers dans `values/` pour adapter la configuration à vos besoins.

---
Généré automatiquement depuis [template-helm](https://github.com/Flowbird-Up/template-helm)
