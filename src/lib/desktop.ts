import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import type { RewriteModeId } from "../data/modes";
import { defaultOutput } from "../data/modes";

export type ProviderId =
  | "offline"
  | "openai"
  | "openrouter"
  | "groq"
  | "gemini"
  | "anthropic"
  | "mistral"
  | "cohere"
  | "xai"
  | "deepseek"
  | "ollama";

export type ProviderSettings = {
  provider: ProviderId;
  model: string;
  apiKey?: string;
  endpoint?: string;
  temperature: number;
  maxTokens: number;
  useOfflineFallback: boolean;
};

export type AppSettings = {
  theme: "system" | "dark" | "light";
  popupTheme: "system" | "dark" | "light";
  accentColor: string;
  launchAtStartup: boolean;
  minimizeToTray: boolean;
  autoReplace: boolean;
  autoCopy: boolean;
  defaultLanguage: string;
  customPrompt: string;
  globalShortcut: string;
  grammarShortcut: string;
  professionalShortcut: string;
  friendlyShortcut: string;
  shorterShortcut: string;
  translateShortcut: string;
  summarizeShortcut: string;
  confidentShortcut: string;
  simplifyShortcut: string;
  provider: ProviderSettings;
};

export type RewriteRequest = {
  input: string;
  mode: RewriteModeId;
  targetLanguage?: string;
  customPrompt?: string;
};

export type RewriteResponse = {
  input: string;
  output: string;
  mode: RewriteModeId;
  provider: ProviderId;
  usedOfflineFallback: boolean;
  characterCount: number;
  elapsedMs: number;
};

export type PopupPayload = RewriteResponse & {
  source: "shortcut" | "tray" | "manual";
  loading?: boolean;
};

export const defaultSettings: AppSettings = {
  theme: "system",
  popupTheme: "dark",
  accentColor: "#8b5cf6",
  launchAtStartup: false,
  minimizeToTray: true,
  autoReplace: true,
  autoCopy: false,
  defaultLanguage: "English",
  customPrompt: "",
  globalShortcut: "Ctrl + Alt + X",
  grammarShortcut: "Ctrl + 1",
  professionalShortcut: "Ctrl + 2",
  friendlyShortcut: "Ctrl + 3",
  shorterShortcut: "Ctrl + 4",
  translateShortcut: "Ctrl + 5",
  summarizeShortcut: "Ctrl + 6",
  confidentShortcut: "Ctrl + 7",
  simplifyShortcut: "Ctrl + 8",
  provider: {
    provider: "offline",
    model: "local-cleanup",
    temperature: 0.35,
    maxTokens: 700,
    useOfflineFallback: true
  }
};

const isTauri = () =>
  typeof window !== "undefined" &&
  (Boolean(window.__TAURI_INTERNALS__) ||
    Boolean(window.__TAURI__) ||
    navigator.userAgent.includes("Tauri") ||
    window.location.protocol === "tauri:" ||
    window.location.hostname === "tauri.localhost");

export async function callCommand<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  if (!isTauri()) {
    return mockCommand<T>(command, args);
  }

  return invoke<T>(command, args);
}

export async function rewriteText(request: RewriteRequest): Promise<RewriteResponse> {
  return callCommand<RewriteResponse>("rewrite_text", { request });
}

export async function copyText(text: string) {
  return callCommand<void>("copy_text", { text });
}

export async function replaceSelectedText(text: string) {
  return callCommand<void>("replace_selected_text", { text });
}

export async function getSettings() {
  return callCommand<AppSettings>("get_settings");
}

export async function saveSettings(settings: AppSettings) {
  return callCommand<AppSettings>("save_settings", { settings });
}

export async function getPopupPayload() {
  return callCommand<PopupPayload | null>("get_popup_payload");
}

export async function hideCurrentWindow() {
  if (!isTauri()) {
    return;
  }

  await callCommand<void>("hide_popup_window").catch(() => getCurrentWindow().hide());
}

export async function startCurrentWindowDrag() {
  if (!isTauri()) {
    return;
  }

  await getCurrentWindow().startDragging();
}

export async function windowAction(action: "minimize" | "maximize" | "close") {
  if (!isTauri()) {
    return;
  }

  const current = getCurrentWindow();
  if (action === "minimize") {
    await current.minimize();
  }
  if (action === "maximize") {
    const maximized = await current.isMaximized();
    if (maximized) {
      await current.unmaximize();
    } else {
      await current.maximize();
    }
  }
  if (action === "close") {
    await callCommand<void>("close_main_window").catch(() => current.close());
  }
}

export async function setWindowTheme(theme: "light" | "dark") {
  if (!isTauri()) {
    return;
  }

  await getCurrentWindow().setTheme(theme);
}

export async function testProviderConnection(settings: ProviderSettings) {
  return callCommand<string>("test_provider_connection", { settings });
}

export async function openProviderGuide(url: string) {
  if (!isTauri()) {
    window.open(url, "_blank", "noopener,noreferrer");
    return;
  }
  return callCommand<void>("open_provider_guide", { url });
}

export function listenToPopupPayload(onPayload: (payload: PopupPayload) => void) {
  if (!isTauri()) {
    return Promise.resolve(() => undefined);
  }

  return listen<PopupPayload>("popup-context", (event) => onPayload(event.payload));
}

export function listenToTrayNavigation(onRoute: (route: string) => void) {
  if (!isTauri()) {
    return Promise.resolve(() => undefined);
  }

  return listen<string>("tray-navigate", (event) => onRoute(event.payload));
}

export function listenToSettingsUpdates(onSettings: (settings: AppSettings) => void) {
  if (!isTauri()) {
    return Promise.resolve(() => undefined);
  }

  return listen<AppSettings>("settings-updated", (event) => onSettings(event.payload));
}

function mockCommand<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  if (command === "get_settings") {
    return Promise.resolve(defaultSettings as T);
  }

  if (command === "save_settings") {
    return Promise.resolve((args?.settings ?? defaultSettings) as T);
  }

  if (command === "get_popup_payload") {
    return Promise.resolve(null as T);
  }

  if (command === "rewrite_text") {
    const request = args?.request as RewriteRequest | undefined;
    if (request?.customPrompt?.trim()) {
      return Promise.reject(new Error("Connect an AI provider to use Custom Prompt."));
    }
    const output = request?.input?.trim()
      ? browserRewriteFallback(request.input, request.mode, request.targetLanguage)
      : defaultOutput;

    return Promise.resolve({
      input: request?.input ?? "",
      output,
      mode: request?.mode ?? "fixGrammar",
      provider: "offline",
      usedOfflineFallback: true,
      characterCount: output.length,
      elapsedMs: 0
    } as T);
  }

  if (command === "test_provider_connection") {
    return Promise.reject(new Error("Provider testing is available in the desktop app."));
  }

  return Promise.resolve(undefined as T);
}

function browserRewriteFallback(input: string, mode: RewriteModeId, targetLanguage?: string) {
  const punctuate = (value: string) => /[.!?]$/.test(value.trim()) ? value.trim() : `${value.trim()}.`;
  const withoutGreeting = (value: string) => value.replace(/^hey(?: there)?[,!]?\s*/i, "");
  let cleaned = input
    .replace(/\b(?:could|should|would) of\b/gi, (value) => value.replace(/ of$/i, " have"))
    .replace(/\bmore better\b/gi, "better")
    .replace(/\bI (?:is|are)\b/g, "I am")
    .replace(/\b(you|we|they) is\b/gi, "$1 are")
    .replace(/\b(he|she|it) are\b/gi, "$1 is")
    .replace(/\bThe team have completed\b/gi, "The team completed")
    .replace(/\bit were\b/gi, "it was")
    .replace(/\bto client\b/gi, "to the client")
    .replace(/\b(ths|thsi)\b/gi, "this")
    .replace(/\bteh\b/gi, "the")
    .replace(/\brecieve\b/gi, "receive")
    .replace(/\bdefinately\b/gi, "definitely")
    .replace(/\balot\b/gi, "a lot")
    .replace(/\bim\b/gi, "I'm")
    .replace(/\bdont\b/gi, "do not")
    .replace(/\bcant\b/gi, "cannot")
    .replace(/\s+([,.!?;:])/g, "$1")
    .replace(/\s+/g, " ")
    .trim();
  cleaned = cleaned.charAt(0).toUpperCase() + cleaned.slice(1);

  if (mode === "professional") {
    cleaned = withoutGreeting(cleaned)
      .replace(/\bI'm\b/g, "I am")
      .replace(/\bI want\b/gi, "I would like")
      .replace(/\bcan you(?: please)?\b/gi, "Could you please")
      .replace(/\bget back to me\b/gi, "respond")
      .replace(/\bASAP\b/g, "as soon as possible");
    return punctuate(cleaned);
  }

  if (mode === "friendly") {
    const body = withoutGreeting(cleaned);
    return punctuate(body.length < 120 ? `Hi! ${body}` : body);
  }

  if (mode === "shorter" || mode === "summarize") {
    cleaned = withoutGreeting(cleaned)
      .replace(/\b(?:basically|actually|to be honest),?\s*/gi, "")
      .replace(/\bin order to\b/gi, "to")
      .replace(/\bdue to the fact that\b/gi, "because");
    const sentences = cleaned.match(/[^.!?]+[.!?]?/g)?.map((sentence) => sentence.trim()) ?? [cleaned];
    const key = sentences.slice(1).find((sentence) => /\b(must|need|will|deadline|risk|recommend|important)\b/i.test(sentence));
    return [sentences[0], key].filter(Boolean).map((sentence) => punctuate(sentence!)).join(" ");
  }

  if (mode === "confident") {
    cleaned = cleaned
      .replace(/^(?:I think|I believe|It seems that|Maybe|Perhaps)\s+/i, "")
      .replace(/\bwe might\b/gi, "we will")
      .replace(/\bshould be able to\b/gi, "can");
  }

  if (mode === "simplify") {
    const simple: Record<string, string> = {
      utilize: "use", approximately: "about", commence: "start", terminate: "end",
      purchase: "buy", assist: "help", "prior to": "before", "in order to": "to"
    };
    for (const [from, to] of Object.entries(simple)) cleaned = cleaned.replace(new RegExp(`\\b${from}\\b`, "gi"), to);
  }

  if (mode === "translate") {
    return `Translation to ${targetLanguage || "the selected language"} requires an AI provider. Original text: ${punctuate(cleaned)}`;
  }

  return punctuate(cleaned);
}
