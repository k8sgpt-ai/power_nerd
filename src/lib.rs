// k8sgpt backend_types as const array

pub const BACKEND_TYPES: &[&str] = &[
    "openai",
    "amazonbedrock",
    "localai",
    "ollama",
    "azureopenai",
    "cohere",
    "amazonsagemaker",
    "google",
    "huggingface",
    "googlevertexai",
    "oci",
    "ibmwatsonxai",
];

pub const FILTER_TYPES: &[&str] = &[
    "None",
    "ReplicaSet",
    "StatefulSet",
    "ValidatingWebhookConfiguration",
    "Service",
    "Ingress",
    "CronJob",
    "Node",
    "MutatingWebhookConfiguration",
    "Pod",
    "Deployment",
    "PersistentVolumeClaim",
    "HorizontalPodAutoScaler",
    "PodDisruptionBudget",
    "NetworkPolicy",
    "Log",
    "GatewayClass",
    "Gateway",
    "HTTPRoute",
];
