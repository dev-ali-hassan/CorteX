import type { ProviderId } from "../lib/desktop";

export type ConnectableProviderId = Exclude<ProviderId, "offline">;

export type ProviderDefinition = {
  id: ConnectableProviderId;
  name: string;
  shortName: string;
  model: string;
  description: string;
  group: "recommended" | "premium" | "local";
  badge?: string;
  guideUrl: string;
  guideLabel: string;
  keyRequired: boolean;
  setupSteps: string[];
};

export const providerDefinitions: ProviderDefinition[] = [
  {
    id: "gemini",
    name: "Google Gemini",
    shortName: "Gemini",
    model: "gemini-2.5-flash",
    description: "Fast, capable models from Google for everyday writing.",
    group: "recommended",
    badge: "Recommended",
    guideUrl: "https://aistudio.google.com/apikey",
    guideLabel: "Google AI Studio",
    keyRequired: true,
    setupSteps: ["Open Google AI Studio", "Sign in with your Google account", "Create an API key", "Copy the key and return to CorteX"]
  },
  {
    id: "groq",
    name: "Groq",
    shortName: "Groq",
    model: "llama-3.3-70b-versatile",
    description: "Extremely fast text generation through GroqCloud.",
    group: "recommended",
    badge: "Fastest",
    guideUrl: "https://console.groq.com/keys",
    guideLabel: "GroqCloud",
    keyRequired: true,
    setupSteps: ["Open the GroqCloud console", "Sign in or create an account", "Create an API key", "Copy the key and return to CorteX"]
  },
  {
    id: "openrouter",
    name: "OpenRouter",
    shortName: "OpenRouter",
    model: "openai/gpt-4o-mini",
    description: "Use many compatible AI models through one provider.",
    group: "recommended",
    badge: "Flexible",
    guideUrl: "https://openrouter.ai/settings/keys",
    guideLabel: "OpenRouter",
    keyRequired: true,
    setupSteps: ["Open OpenRouter settings", "Sign in to your account", "Create a new API key", "Copy the key and return to CorteX"]
  },
  {
    id: "deepseek",
    name: "DeepSeek",
    shortName: "DeepSeek",
    model: "deepseek-v4-flash",
    description: "High-value, capable models for fast everyday rewriting.",
    group: "recommended",
    badge: "Great value",
    guideUrl: "https://platform.deepseek.com/api_keys",
    guideLabel: "DeepSeek Platform",
    keyRequired: true,
    setupSteps: ["Open the DeepSeek Platform", "Sign in to your account", "Create an API key", "Copy the key and return to CorteX"]
  },
  {
    id: "openai",
    name: "OpenAI",
    shortName: "OpenAI",
    model: "gpt-4o-mini",
    description: "OpenAI models for polished, reliable writing.",
    group: "premium",
    guideUrl: "https://platform.openai.com/api-keys",
    guideLabel: "OpenAI Platform",
    keyRequired: true,
    setupSteps: ["Open the OpenAI Platform", "Sign in to your account", "Create a secret API key", "Copy the key and return to CorteX"]
  },
  {
    id: "anthropic",
    name: "Anthropic Claude",
    shortName: "Claude",
    model: "claude-3-5-haiku-latest",
    description: "Claude models with strong language and writing quality.",
    group: "premium",
    guideUrl: "https://platform.claude.com/settings/keys",
    guideLabel: "Claude Console",
    keyRequired: true,
    setupSteps: ["Open the Claude Console", "Sign in to your account", "Create an API key", "Copy the key and return to CorteX"]
  },
  {
    id: "mistral",
    name: "Mistral AI",
    shortName: "Mistral",
    model: "mistral-small-latest",
    description: "Fast European models with strong multilingual writing.",
    group: "premium",
    guideUrl: "https://console.mistral.ai/api-keys",
    guideLabel: "Mistral Studio",
    keyRequired: true,
    setupSteps: ["Open Mistral Studio", "Sign in to your account", "Create an API key", "Copy the key and return to CorteX"]
  },
  {
    id: "cohere",
    name: "Cohere",
    shortName: "Cohere",
    model: "command-a-plus-05-2026",
    description: "Enterprise-focused language models for polished text tasks.",
    group: "premium",
    guideUrl: "https://dashboard.cohere.com/api-keys",
    guideLabel: "Cohere Dashboard",
    keyRequired: true,
    setupSteps: ["Open the Cohere Dashboard", "Sign in to your account", "Create an API key", "Copy the key and return to CorteX"]
  },
  {
    id: "xai",
    name: "xAI",
    shortName: "xAI",
    model: "grok-4.3",
    description: "Grok models with fast, precise instruction following.",
    group: "premium",
    guideUrl: "https://console.x.ai/team/default/api-keys",
    guideLabel: "xAI Console",
    keyRequired: true,
    setupSteps: ["Open the xAI Console", "Sign in to your account", "Create an API key", "Copy the key and return to CorteX"]
  },
  {
    id: "ollama",
    name: "Ollama Local",
    shortName: "Ollama",
    model: "llama3.1",
    description: "Run supported models locally on your own computer.",
    group: "local",
    badge: "Local",
    guideUrl: "https://ollama.com/download",
    guideLabel: "Ollama",
    keyRequired: false,
    setupSteps: ["Install Ollama for Windows", "Start the Ollama application", "Install the llama3.1 model", "Return to CorteX to verify it"]
  }
];

export const providerLabels: Record<ProviderId, string> = {
  offline: "Offline utilities",
  openai: "OpenAI",
  openrouter: "OpenRouter",
  groq: "Groq",
  gemini: "Google Gemini",
  anthropic: "Anthropic Claude",
  mistral: "Mistral AI",
  cohere: "Cohere",
  xai: "xAI",
  deepseek: "DeepSeek",
  ollama: "Ollama local"
};

export const providerModels: Record<ProviderId, string> = {
  offline: "local-cleanup",
  openai: "gpt-4o-mini",
  openrouter: "openai/gpt-4o-mini",
  groq: "llama-3.3-70b-versatile",
  gemini: "gemini-2.5-flash",
  anthropic: "claude-3-5-haiku-latest",
  mistral: "mistral-small-latest",
  cohere: "command-a-plus-05-2026",
  xai: "grok-4.3",
  deepseek: "deepseek-v4-flash",
  ollama: "llama3.1"
};

export function getProviderDefinition(id: ConnectableProviderId) {
  return providerDefinitions.find((provider) => provider.id === id) ?? providerDefinitions[0];
}
