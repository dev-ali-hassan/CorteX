import { useEffect, useRef, useState } from "react";
import type { ChangeEvent, KeyboardEvent, ReactNode } from "react";
import clsx from "clsx";
import {
  ChevronDown,
  CheckCircle2,
  Copy,
  ChevronRight,
  FileText,
  FileUp,
  Globe2,
  Keyboard,
  Plug,
  Search,
  Settings,
  SlidersHorizontal,
  Sparkles,
  Trash2,
  Wand2,
  Zap,
  X,
  Minus,
  Square,
  Power,
  Monitor,
  Moon,
  Sun
} from "lucide-react";
import {
  copyText,
  defaultSettings,
  getSettings,
  listenToTrayNavigation,
  replaceSelectedText,
  rewriteText,
  saveSettings,
  setWindowTheme,
  testProviderConnection,
  windowAction,
  type AppSettings
} from "./lib/desktop";
import { extractTextFromDocument } from "./lib/documentImport";
import { defaultInput, defaultOutput, rewriteModes, type RewriteModeId } from "./data/modes";
import { providerLabels } from "./data/providers";
import { ProviderWizard } from "./components/ProviderWizard";

type ViewKey = "rewrite" | "settings";
type ProviderConnectionState = "disconnected" | "checking" | "connected" | "error";

const visibleRewriteModes = rewriteModes.filter(
  (item) => item.id !== "shorter" && item.id !== "confident"
);

const PROVIDER_ONBOARDING_KEY = "cortex.provider-onboarding.v1";

function providerOnboardingWasSeen() {
  try {
    return window.localStorage.getItem(PROVIDER_ONBOARDING_KEY) === "complete";
  } catch {
    return false;
  }
}

function rememberProviderOnboarding() {
  try {
    window.localStorage.setItem(PROVIDER_ONBOARDING_KEY, "complete");
  } catch {
    // Provider settings are still preserved by the desktop database.
  }
}

function App() {
  const [view, setView] = useState<ViewKey>("rewrite");
  const [input, setInput] = useState(defaultInput);
  const [output, setOutput] = useState(defaultOutput);
  const [mode, setMode] = useState<RewriteModeId>("fixGrammar");
  const [settings, setSettings] = useState<AppSettings>(defaultSettings);
  const [status, setStatus] = useState("Ready");
  const [loading, setLoading] = useState(false);
  const [rewriteElapsedMs, setRewriteElapsedMs] = useState<number | null>(null);
  const [copied, setCopied] = useState(false);
  const [settingsJumpTarget, setSettingsJumpTarget] = useState<string | null>(null);
  const [providerConnection, setProviderConnection] = useState<ProviderConnectionState>("disconnected");
  const [providerConnectionMessage, setProviderConnectionMessage] = useState("Select and test a provider.");
  const [providerWizardOpen, setProviderWizardOpen] = useState(
    () => import.meta.env.DEV && new URLSearchParams(window.location.search).has("providerSetup")
  );
  const [providerWizardOnboarding, setProviderWizardOnboarding] = useState(false);
  const providerCheckId = useRef(0);
  const [systemPrefersDark, setSystemPrefersDark] = useState(() =>
    window.matchMedia("(prefers-color-scheme: dark)").matches
  );

  useEffect(() => {
    const colorScheme = window.matchMedia("(prefers-color-scheme: dark)");
    const syncSystemTheme = () => setSystemPrefersDark(colorScheme.matches);

    syncSystemTheme();
    colorScheme.addEventListener("change", syncSystemTheme);
    return () => colorScheme.removeEventListener("change", syncSystemTheme);
  }, []);

  useEffect(() => {
    if (
      settings.provider.provider === "offline" ||
      settings.provider.model.trim().toLowerCase() === "local-cleanup"
    ) {
      setProviderConnection("disconnected");
      setProviderConnectionMessage("Select an AI provider to connect.");
    } else {
      setProviderConnection("checking");
      setProviderConnectionMessage("Checking the provider connection...");
    }
    const timer = window.setTimeout(() => {
      void verifyProviderConnection(settings.provider);
    }, 350);
    return () => window.clearTimeout(timer);
  }, [
    settings.provider.provider,
    settings.provider.model,
    settings.provider.apiKey,
    settings.provider.endpoint
  ]);

  useEffect(() => {
    getSettings()
      .then((value) => {
        if (value && typeof value === "object") {
          const theme = ["system", "dark", "light"].includes(value.theme) ? value.theme : "system";
          const loadedSettings = { ...value, theme };
          setSettings(loadedSettings);
          if (!providerOnboardingWasSeen() && loadedSettings.provider.provider === "offline") {
            setProviderWizardOnboarding(true);
            setProviderWizardOpen(true);
          }
        }
      })
      .catch(() => {
        setSettings(defaultSettings);
        if (!providerOnboardingWasSeen()) {
          setProviderWizardOnboarding(true);
          setProviderWizardOpen(true);
        }
      });

    listenToTrayNavigation((route) => {
      if (route === "settings" || route === "rewrite") {
        setView(route);
      }
    }).then((unlisten) => () => unlisten());
  }, []);

  useEffect(() => {
    if (view !== "settings" || !settingsJumpTarget) {
      return;
    }

    let attempts = 0;
    const timers: number[] = [];
    const scrollToTarget = () => {
      attempts += 1;
      const target = document.getElementById(settingsJumpTarget);
      const scroller = document.querySelector<HTMLElement>(".settings-panel");

      if (target && scroller) {
        const top = target.offsetTop - scroller.offsetTop;
        scroller.scrollTo({ top: Math.max(0, top - 10), behavior: "smooth" });
        setSettingsJumpTarget(null);
        return;
      }

      if (target) {
        target.scrollIntoView({ behavior: "smooth", block: "start" });
        setSettingsJumpTarget(null);
        return;
      }

      if (attempts < 8) {
        timers.push(window.setTimeout(scrollToTarget, 80));
      } else {
        setSettingsJumpTarget(null);
      }
    };

    const frame = window.requestAnimationFrame(scrollToTarget);

    return () => {
      window.cancelAnimationFrame(frame);
      timers.forEach(window.clearTimeout);
    };
  }, [settingsJumpTarget, view]);

  async function runRewrite(nextMode = mode, customPrompt?: string) {
    const text = input.trim();
    if (!text) {
      setStatus("Add text first");
      return;
    }

    setLoading(true);
    setCopied(false);
    setRewriteElapsedMs(null);
    setStatus("Rewriting");
    try {
      const response = await rewriteText({
        input: text,
        mode: nextMode,
        targetLanguage: settings.defaultLanguage,
        customPrompt
      });
      setOutput(response.output);
      setRewriteElapsedMs(response.elapsedMs);
      setStatus(response.usedOfflineFallback ? "Offline rewrite ready" : "AI rewrite ready");
      if (settings.autoCopy) {
        await copyText(response.output);
        setStatus("Rewritten and copied");
      }
    } catch (error) {
      setStatus(error instanceof Error ? error.message : "Rewrite failed");
    } finally {
      setLoading(false);
    }
  }

  async function handleCopy() {
    if (!output.trim()) {
      return;
    }
    await copyText(output);
    setCopied(true);
    setStatus("Copied");
    window.setTimeout(() => setCopied(false), 2000);
  }

  async function handleReplace() {
    if (!output.trim()) {
      return;
    }
    await replaceSelectedText(output);
    setStatus("Replaced selected text");
  }

  async function handleDocumentImport(file: File) {
    setLoading(true);
    setCopied(false);
    setStatus("Importing document");
    try {
      const text = await extractTextFromDocument(file);
      if (!text.trim()) {
        setStatus("No readable text found");
        return;
      }
      setInput(text);
      setOutput("");
      setRewriteElapsedMs(null);
      setStatus(`Imported ${file.name}`);
    } catch (error) {
      setStatus(error instanceof Error ? error.message : "Could not import file");
    } finally {
      setLoading(false);
    }
  }

  const mainTitle = {
    rewrite: "Quick Rewrite",
    settings: "Settings"
  }[view];
  const isOfflineModel =
    settings.provider.provider === "offline" ||
    settings.provider.model.trim().toLowerCase() === "local-cleanup";
  const hasConnectedProvider = !isOfflineModel && providerConnection === "connected";
  const sidebarProviderName = isOfflineModel
    ? "No Provider Selected"
    : providerLabels[settings.provider.provider] || "Custom Provider";
  const sidebarProviderStatus = {
    disconnected: "Not Connected",
    checking: "Checking...",
    connected: "Connected",
    error: "Connection Failed"
  }[providerConnection];
  const savedTheme = ["system", "dark", "light"].includes(settings.theme) ? settings.theme : "system";
  const activeTheme = savedTheme === "system" ? (systemPrefersDark ? "dark" : "light") : savedTheme;

  useEffect(() => {
    const windowTheme = activeTheme === "light" ? "light" : "dark";
    document.documentElement.style.colorScheme = windowTheme;
    void setWindowTheme(windowTheme).catch(() => undefined);
  }, [activeTheme]);

  function openShortcutSettings() {
    setSettingsJumpTarget("shortcuts-section");
    setView("settings");
  }

  function openProviderSettings() {
    setProviderWizardOnboarding(false);
    setProviderWizardOpen(true);
  }

  function closeProviderWizard() {
    if (providerWizardOnboarding) {
      rememberProviderOnboarding();
    }
    setProviderWizardOnboarding(false);
    setProviderWizardOpen(false);
  }

  function handleThemeChange(theme: AppSettings["theme"]) {
    const nextSettings = { ...settings, theme };
    setSettings(nextSettings);
    void saveSettings(nextSettings)
      .then(() => setStatus("Theme saved"))
      .catch(() => setStatus("Theme saved locally"));
  }

  async function verifyProviderConnection(providerSettings = settings.provider) {
    const checkId = ++providerCheckId.current;
    if (
      providerSettings.provider === "offline" ||
      providerSettings.model.trim().toLowerCase() === "local-cleanup"
    ) {
      setProviderConnection("disconnected");
      setProviderConnectionMessage("Select an AI provider to connect.");
      return;
    }

    setProviderConnection("checking");
    setProviderConnectionMessage("Checking the provider connection...");
    try {
      await testProviderConnection(providerSettings);
      if (providerCheckId.current === checkId) {
        setProviderConnection("connected");
        setProviderConnectionMessage("Connection verified successfully.");
      }
    } catch (error) {
      if (providerCheckId.current === checkId) {
        setProviderConnection("error");
        setProviderConnectionMessage(error instanceof Error ? error.message : "Could not connect to the provider.");
      }
    }
  }

  async function connectProvider(providerSettings: AppSettings["provider"]) {
    const nextSettings = { ...settings, provider: providerSettings };
    const saved = await saveSettings(nextSettings);
    setSettings(saved && typeof saved === "object" ? saved : nextSettings);
    setProviderConnection("connected");
    setProviderConnectionMessage("Connection verified successfully.");
    setStatus(`${providerLabels[providerSettings.provider]} connected`);
    rememberProviderOnboarding();
  }

  return (
    <main className="desktop-window" data-theme={activeTheme} aria-label="CorteX desktop app">
      <TitleControls />
      <aside className="sidebar">
        <BrandBlock />
        <nav className="nav-list" aria-label="Primary">
          <NavButton
            active={view === "rewrite"}
            icon={Wand2}
            label="Quick Rewrite"
            onClick={() => setView("rewrite")}
          />
          <NavButton
            active={view === "settings"}
            icon={Settings}
            label="Settings"
            onClick={() => setView("settings")}
          />
        </nav>
        <div className="sidebar-separator" />
        <div className="sidebar-group-label">Shortcut</div>
        <button
          className="shortcut-card"
          type="button"
          aria-label="Open shortcut settings"
          onClick={openShortcutSettings}
        >
          <span className="shortcut-card-main">
            <span className="shortcut-icon" aria-hidden="true">
              <Keyboard size={22} />
            </span>
            <strong>Quick Rewrite</strong>
          </span>
          <span className="shortcut-keys" aria-label={`Current shortcut ${settings.globalShortcut}`}>
            {settings.globalShortcut.split("+").map((part, index, parts) => (
              <span key={`${part}-${index}`}>
                <kbd>{part.trim()}</kbd>
                {index < parts.length - 1 && <b>+</b>}
              </span>
            ))}
          </span>
          <span className="shortcut-card-footer">
            <small>Press anytime</small>
            <ChevronRight size={18} aria-hidden="true" />
          </span>
        </button>
        <div className="sidebar-separator" />
        <button
          className="model-card"
          type="button"
          aria-label="Open AI provider settings"
          onClick={openProviderSettings}
        >
          <span className="model-icon" aria-hidden="true">
            {hasConnectedProvider ? <Sparkles size={22} /> : <Plug size={22} />}
          </span>
          <span className="model-card-copy">
            <small>AI Provider</small>
            <strong>{sidebarProviderName}</strong>
            <em className={clsx(!hasConnectedProvider && "not-connected", providerConnection === "error" && "connection-error")}>
              {sidebarProviderStatus}
            </em>
          </span>
          <ChevronRight className="model-card-arrow" size={17} aria-hidden="true" />
        </button>
        <div className="sidebar-version">
          <span>v1.0.1</span>
        </div>
      </aside>

      <section className="workspace" aria-labelledby="workspace-title">
        <header className="workspace-header" data-tauri-drag-region>
          <div>
            <h1 id="workspace-title">{mainTitle}</h1>
            <p>{view === "rewrite" ? "Rewrite, improve, and perfect your text instantly." : viewSubtitle(view)}</p>
          </div>
        </header>

        {view === "rewrite" && (
          <QuickRewrite
            input={input}
            output={output}
            mode={mode}
            loading={loading}
            elapsedMs={rewriteElapsedMs}
            status={status}
            copied={copied}
            setInput={setInput}
            setMode={(nextMode) => {
              setMode(nextMode);
              void runRewrite(nextMode);
            }}
            onRewrite={() => runRewrite()}
            onCustomRewrite={(instruction) => runRewrite(mode, instruction)}
            onCopy={handleCopy}
            onReplace={handleReplace}
            onDocumentImport={handleDocumentImport}
          />
        )}
        {view === "settings" && (
          <SettingsView
            settings={settings}
            providerConnection={providerConnection}
            providerConnectionMessage={providerConnectionMessage}
            onOpenProviderWizard={openProviderSettings}
            onThemeChange={handleThemeChange}
            onChange={async (nextSettings) => {
              const previousSettings = settings;
              setSettings(nextSettings);
              try {
                const saved = await saveSettings(nextSettings);
                if (saved && typeof saved === "object") {
                  setSettings(saved);
                }
                setStatus("Settings saved");
              } catch (error) {
                setSettings(previousSettings);
                setStatus(error instanceof Error ? error.message : "Could not apply this setting");
              }
            }}
          />
        )}
      </section>
      {providerWizardOpen && (
        <ProviderWizard
          initialSettings={settings.provider}
          showScreenSkip={providerWizardOnboarding}
          onClose={closeProviderWizard}
          onConnected={connectProvider}
        />
      )}
    </main>
  );
}

function viewSubtitle(view: ViewKey) {
  return view === "rewrite"
    ? "Turn rough text into polished writing."
    : "Choose providers, shortcuts, and behavior for the desktop assistant.";
}

function TitleControls() {
  return (
    <div className="title-controls" aria-label="Window controls">
      <button type="button" aria-label="Minimize" onClick={() => windowAction("minimize")}>
        <Minus size={18} aria-hidden="true" />
      </button>
      <button type="button" aria-label="Maximize" onClick={() => windowAction("maximize")}>
        <Square size={16} aria-hidden="true" />
      </button>
      <button type="button" aria-label="Close to tray" onClick={() => windowAction("close")}>
        <X size={20} aria-hidden="true" />
      </button>
    </div>
  );
}

function BrandBlock() {
  return (
    <div className="brand-block" data-tauri-drag-region>
      <LogoMark small />
      <div>
        <div className="brand-name">
          <span>CorteX</span>
        </div>
        <small>AI Writing Assistant</small>
        <span className="brand-status">
          <i aria-hidden="true" />
          Ready
        </span>
      </div>
    </div>
  );
}

function LogoMark({ small = false }: { small?: boolean }) {
  return (
    <span className={clsx("logo-mark", small && "logo-mark-small")} aria-hidden="true">
      <img src="/cortex-icon.png" alt="" draggable={false} />
    </span>
  );
}

function NavButton({
  active,
  icon: Icon,
  label,
  onClick
}: {
  active: boolean;
  icon: typeof Wand2;
  label: string;
  onClick: () => void;
}) {
  return (
    <button className={clsx("nav-button", active && "active")} type="button" onClick={onClick}>
      <Icon size={28} strokeWidth={2.2} aria-hidden="true" />
      <span>{label}</span>
    </button>
  );
}

function countWords(value: string) {
  const words = value.trim().match(/\S+/g);
  return words ? words.length : 0;
}

function formatWordCount(count: number) {
  return `${count} ${count === 1 ? "word" : "words"}`;
}

function formatElapsed(milliseconds: number | null) {
  if (milliseconds === null) {
    return "--";
  }
  if (milliseconds < 1000) {
    return `${milliseconds}ms`;
  }
  return `${(milliseconds / 1000).toFixed(2)}s`;
}

function QuickRewrite({
  input,
  output,
  mode,
  loading,
  elapsedMs,
  status,
  copied,
  setInput,
  setMode,
  onRewrite,
  onCustomRewrite,
  onCopy,
  onReplace,
  onDocumentImport
}: {
  input: string;
  output: string;
  mode: RewriteModeId;
  loading: boolean;
  elapsedMs: number | null;
  status: string;
  copied: boolean;
  setInput: (value: string) => void;
  setMode: (value: RewriteModeId) => void;
  onRewrite: () => void;
  onCustomRewrite: (instruction: string) => Promise<void>;
  onCopy: () => void;
  onReplace: () => void;
  onDocumentImport: (file: File) => Promise<void>;
}) {
  const selectedMode = rewriteModes.find((item) => item.id === mode);
  const inputWords = countWords(input);
  const outputWords = countWords(output);
  const fileInputRef = useRef<HTMLInputElement>(null);
  const [customPromptOpen, setCustomPromptOpen] = useState(false);
  const [customInstruction, setCustomInstruction] = useState("");
  const [customPromptActive, setCustomPromptActive] = useState(false);

  async function handleFileChange(event: ChangeEvent<HTMLInputElement>) {
    const file = event.target.files?.[0];
    if (!file) {
      return;
    }
    await onDocumentImport(file);
    event.target.value = "";
  }

  async function handleCustomRewrite() {
    const instruction = customInstruction.trim();
    if (!instruction) {
      return;
    }
    setCustomPromptActive(true);
    setCustomPromptOpen(false);
    await onCustomRewrite(instruction);
  }

  function handleModeSelect(nextMode: RewriteModeId) {
    setCustomPromptActive(false);
    setMode(nextMode);
  }

  function handleRewriteAgain() {
    const instruction = customInstruction.trim();
    if (customPromptActive && instruction) {
      void onCustomRewrite(instruction);
      return;
    }
    onRewrite();
  }

  return (
    <div className="rewrite-surface">
      <section className="text-panel original-panel" aria-labelledby="input-label">
        <header className="rewrite-card-heading">
          <span className="rewrite-heading-main">
            <FileText size={20} aria-hidden="true" />
            <label id="input-label" htmlFor="input-text">
              Original text
            </label>
          </span>
        </header>
        <div className="editor-shell">
          <textarea
            id="input-text"
            value={input}
            onChange={(event) => setInput(event.target.value)}
            spellCheck
            placeholder="Paste or type text to rewrite..."
            aria-describedby="rewrite-status"
          />
        </div>
        <div className="input-meta-row">
          <span className="language-chip">
            <Globe2 size={18} aria-hidden="true" />
            English
          </span>
          <span className="word-count">{formatWordCount(inputWords)}</span>
          <input
            ref={fileInputRef}
            className="file-input"
            type="file"
            accept=".txt,.md,.markdown,.csv,.json,.log,.rtf,.docx,.pdf,application/pdf,application/vnd.openxmlformats-officedocument.wordprocessingml.document"
            onChange={handleFileChange}
          />
          <button className="ghost-tool" type="button" onClick={() => fileInputRef.current?.click()} disabled={loading}>
            <FileUp size={18} aria-hidden="true" />
            <span>Import file</span>
          </button>
          <button className="ghost-tool" type="button" onClick={() => setInput("")}>
            <Trash2 size={18} aria-hidden="true" />
            <span>Clear</span>
          </button>
        </div>
      </section>

      <section className="mode-section" aria-labelledby="mode-label">
        <div className="mode-section-header">
          <h2 id="mode-label">Rewrite mode</h2>
        </div>
        <div className="mode-grid">
          {visibleRewriteModes.map((item) => (
            <button
              className={clsx("mode-tile", !customPromptActive && mode === item.id && "selected")}
              type="button"
              key={item.id}
              onClick={() => handleModeSelect(item.id)}
              aria-pressed={!customPromptActive && mode === item.id}
              title={item.description}
            >
              <item.icon size={34} strokeWidth={2} aria-hidden="true" />
              <span>{item.label}</span>
            </button>
          ))}
          <button
            className={clsx("prompt-trigger mode-prompt-trigger", (customPromptOpen || customPromptActive) && "active")}
            type="button"
            onClick={() => setCustomPromptOpen(true)}
            aria-expanded={customPromptOpen}
            aria-pressed={customPromptActive}
            title="Rewrite with your own AI instruction"
          >
            <span className="mode-prompt-icon" aria-hidden="true">
              <SlidersHorizontal size={21} strokeWidth={2.1} />
            </span>
            <span>Custom Prompt</span>
          </button>
        </div>
        {customPromptOpen && (
          <div className="custom-prompt-overlay" role="presentation" onMouseDown={() => setCustomPromptOpen(false)}>
            <section
              className="custom-prompt-panel"
              role="dialog"
              aria-modal="true"
              aria-labelledby="custom-prompt-title"
              onMouseDown={(event) => event.stopPropagation()}
            >
              <header>
                <div>
                  <span>AI instruction</span>
                  <h3 id="custom-prompt-title">Custom Prompt</h3>
                </div>
                <button type="button" aria-label="Close custom prompt" onClick={() => setCustomPromptOpen(false)}>
                  <X size={20} aria-hidden="true" />
                </button>
              </header>
              <label htmlFor="custom-instruction">Instruction</label>
              <textarea
                id="custom-instruction"
                value={customInstruction}
                onChange={(event) => setCustomInstruction(event.target.value)}
                placeholder="Example: Make this warm, professional, and suitable for a client email."
                autoFocus
              />
              <p>Requires a connected AI provider.</p>
              <footer>
                <button className="custom-prompt-cancel" type="button" onClick={() => setCustomPromptOpen(false)}>
                  Cancel
                </button>
                <button
                  className="custom-prompt-run"
                  type="button"
                  onClick={() => void handleCustomRewrite()}
                  disabled={!customInstruction.trim() || loading}
                >
                  <Sparkles size={18} aria-hidden="true" />
                  Rewrite
                </button>
              </footer>
            </section>
          </div>
        )}
      </section>

      <section className="text-panel output-panel" aria-labelledby="output-label">
        <header className="rewrite-card-heading">
          <span className="rewrite-heading-main improved">
            <Sparkles size={20} aria-hidden="true" />
            <label id="output-label" htmlFor="output-text">
              Improved text
            </label>
          </span>
          <span className="success-chip">
            <CheckCircle2 size={18} aria-hidden="true" />
            {customPromptActive ? "Custom Prompt" : selectedMode?.label ?? "Rewrite"} ready
          </span>
        </header>
        <div className={clsx("editor-shell", loading && "editor-shell-loading")}>
          {loading ? (
            <div className="rewrite-loader" role="status" aria-live="polite">
              <span className="rewrite-loader-title">
                <Sparkles size={18} aria-hidden="true" />
                Rewriting...
              </span>
              <span className="rewrite-loader-bar" aria-hidden="true">
                <span />
              </span>
            </div>
          ) : (
            <textarea id="output-text" value={output} readOnly placeholder="Your rewritten text will appear here..." />
          )}
        </div>
        <div className="output-meta-row">
          <div className="output-stats">
            <span>
              <CheckCircle2 size={18} aria-hidden="true" />
              {status}
            </span>
            <span>{formatElapsed(elapsedMs)}</span>
          </div>
          <span className="word-count output-word-count">{formatWordCount(outputWords)}</span>
          <div className="output-actions">
            <button className={clsx("primary-action", copied && "copied")} type="button" onClick={onCopy}>
              {copied ? <CheckCircle2 size={20} aria-hidden="true" /> : <Copy size={23} aria-hidden="true" />}
              <span>{copied ? "Copied" : "Copy"}</span>
            </button>
            <button
              className="primary-action rewrite-output-action"
              type="button"
              onClick={handleRewriteAgain}
              disabled={loading}
            >
              <Zap size={18} aria-hidden="true" />
              <span>{loading ? "Rewriting" : "Rewrite"}</span>
            </button>
          </div>
        </div>
      </section>
    </div>
  );
}

function SettingsView({
  settings,
  providerConnection,
  providerConnectionMessage,
  onOpenProviderWizard,
  onChange,
  onThemeChange
}: {
  settings: AppSettings;
  providerConnection: ProviderConnectionState;
  providerConnectionMessage: string;
  onOpenProviderWizard: () => void;
  onChange: (settings: AppSettings) => void | Promise<void>;
  onThemeChange: (theme: AppSettings["theme"]) => void;
}) {
  const [settingsSearch, setSettingsSearch] = useState("");
  const normalizedSearch = settingsSearch.trim().toLowerCase();
  const sectionMatches = (keywords: string[]) =>
    !normalizedSearch || keywords.some((keyword) => keyword.toLowerCase().includes(normalizedSearch));
  const visibleSections = [
    sectionMatches(["ai", "provider", "api key", "test connection", "gemini", "groq", "openai", "openrouter", "anthropic", "mistral", "cohere", "xai", "grok", "deepseek", "ollama"]),
    sectionMatches(["general", "startup", "auto copy", "auto replace", "behavior"]),
    sectionMatches(["shortcuts", "floating window", "grammar", "professional", "friendly", "shorter", "translate", "summarize", "confident", "simplify"]),
    sectionMatches(["appearance", "theme", "dark", "light", "system", "windows", "recommended"]),
    sectionMatches(["privacy", "stored locally", "analytics", "cloud storage"])
  ];
  const [showAI, showGeneral, showShortcuts, showAppearance, showPrivacy] = visibleSections;
  const hasLaterSection = (index: number) => visibleSections.slice(index + 1).some(Boolean);

  function update(next: Partial<AppSettings>) {
    return onChange({ ...settings, ...next });
  }

  return (
    <div className="settings-shell">
      <label className="settings-search" aria-label="Search settings">
        <Search size={18} aria-hidden="true" />
        <input
          value={settingsSearch}
          placeholder="Search Settings..."
          onChange={(event) => setSettingsSearch(event.target.value)}
        />
      </label>
      <section className="settings-panel settings-stack">
        {showAI && (
        <div className="settings-section settings-card" id="ai-section">
          <h2>AI</h2>
          <div className="settings-provider-overview">
            <span className="settings-provider-icon" aria-hidden="true">
              {providerConnection === "connected" ? <Sparkles size={23} /> : <Plug size={23} />}
            </span>
            <span>
              <small>Current provider</small>
              <strong>{settings.provider.provider === "offline" ? "No Provider Selected" : providerLabels[settings.provider.provider]}</strong>
              <em className={clsx("provider-test-status", providerConnection)}>{providerConnectionMessage}</em>
            </span>
            <button className="test-connection-button" type="button" onClick={onOpenProviderWizard}>
              {settings.provider.provider === "offline" ? "Connect Provider" : "Manage Provider"}
              <ChevronRight size={17} />
            </button>
          </div>
        </div>
        )}

        {showGeneral && (
        <div className="settings-section settings-card">
          <h2>General</h2>
          <ToggleRow
            label="Launch at startup"
            description="Open CorteX automatically when you log in"
            icon={<Power size={20} aria-hidden="true" />}
            checked={settings.launchAtStartup}
            onChange={(launchAtStartup) => update({ launchAtStartup })}
          />
          <ToggleRow
            label="Auto-copy result"
            description="Copy the improved text to the clipboard automatically"
            icon={<Copy size={20} aria-hidden="true" />}
            checked={settings.autoCopy}
            onChange={(autoCopy) => update({ autoCopy })}
          />
          <ToggleRow
            label="Auto-replace selection"
            description="Replace selected text with the improved result"
            icon={<Wand2 size={20} aria-hidden="true" />}
            checked={settings.autoReplace}
            onChange={(autoReplace) => update({ autoReplace })}
          />
          <ToggleRow
            label="Minimize to tray"
            description="Keep CorteX running in the system tray when you close it"
            icon={<Minus size={20} aria-hidden="true" />}
            checked={settings.minimizeToTray}
            onChange={(minimizeToTray) => update({ minimizeToTray })}
          />
        </div>
        )}

        {showShortcuts && (
        <div className="settings-section settings-card" id="shortcuts-section">
          <h2>Shortcuts</h2>
          <ShortcutRecorder label="Floating Window" value={settings.globalShortcut} onChange={(globalShortcut) => update({ globalShortcut })} />
          <ShortcutRecorder label="Grammar" value={settings.grammarShortcut} onChange={(grammarShortcut) => update({ grammarShortcut })} />
          <ShortcutRecorder
            label="Professional"
            value={settings.professionalShortcut}
            onChange={(professionalShortcut) => update({ professionalShortcut })}
          />
          <ShortcutRecorder label="Friendly" value={settings.friendlyShortcut} onChange={(friendlyShortcut) => update({ friendlyShortcut })} />
          <ShortcutRecorder label="Shorter" value={settings.shorterShortcut} onChange={(shorterShortcut) => update({ shorterShortcut })} />
          <ShortcutRecorder label="Translate" value={settings.translateShortcut} onChange={(translateShortcut) => update({ translateShortcut })} />
          <ShortcutRecorder label="Summarize" value={settings.summarizeShortcut} onChange={(summarizeShortcut) => update({ summarizeShortcut })} />
          <ShortcutRecorder label="Confident" value={settings.confidentShortcut} onChange={(confidentShortcut) => update({ confidentShortcut })} />
          <ShortcutRecorder label="Simplify" value={settings.simplifyShortcut} onChange={(simplifyShortcut) => update({ simplifyShortcut })} />
        </div>
        )}

        {showAppearance && (
        <div className="settings-section settings-card">
          <h2>Appearance</h2>
          <div className="theme-options" role="radiogroup" aria-label="Theme">
            <span className="theme-options-label">Theme</span>
            {[
              { value: "dark", label: "Dark", description: "Always use dark appearance.", icon: Moon },
              { value: "light", label: "Light", description: "Always use light appearance.", icon: Sun },
              {
                value: "system",
                label: "System",
                description: "Automatically follows your Windows appearance.",
                icon: Monitor,
                recommended: true
              }
            ].map(({ value, label, description, icon: ThemeIcon, recommended }) => (
              <button
                className={clsx("theme-choice", settings.theme === value && "selected")}
                key={value}
                type="button"
                aria-pressed={settings.theme === value}
                onClick={() => onThemeChange(value as AppSettings["theme"])}
              >
                <span className={clsx("theme-preview", `theme-preview-${value}`)} aria-hidden="true">
                  <span className="theme-preview-bar" />
                  <span className="theme-preview-window">
                    <i />
                    <i />
                    <i />
                    <i />
                  </span>
                </span>
                <span className="theme-choice-heading">
                  <ThemeIcon size={16} aria-hidden="true" />
                  <span className="theme-choice-label">{label}</span>
                  {recommended && <small>Recommended</small>}
                </span>
                <span className="theme-choice-description">{description}</span>
              </button>
            ))}
          </div>
        </div>
        )}

        {!visibleSections.some(Boolean) && (
          <div className="settings-empty-state">
            <Search size={20} aria-hidden="true" />
            <span>No settings found</span>
          </div>
        )}
      </section>
    </div>
  );
}

function ShortcutRecorder({
  label,
  value,
  onChange
}: {
  label: string;
  value: string;
  onChange: (value: string) => void;
}) {
  const [recording, setRecording] = useState(false);
  const [error, setError] = useState("");

  function handleKeyDown(event: KeyboardEvent<HTMLButtonElement>) {
    if (!recording) {
      return;
    }

    event.preventDefault();
    event.stopPropagation();

    const shortcut = formatShortcut(event);
    if (shortcut) {
      if (isUnsafeRecordedShortcut(shortcut)) {
        setError("Add Alt, Shift, or Win, or use Ctrl with a number.");
        return;
      }
      setError("");
      onChange(shortcut);
      setRecording(false);
    }
  }

  return (
    <div className="shortcut-recorder">
      <div>
        <span>{label}</span>
        <small>Current: {value}</small>
        {error && <em className="shortcut-error">{error}</em>}
      </div>
      <button
        type="button"
        className={clsx(recording && "recording")}
        onClick={() => {
          setError("");
          setRecording(true);
        }}
        onKeyDown={handleKeyDown}
        onBlur={() => setRecording(false)}
      >
        {recording ? (
          "Press shortcut"
        ) : (
          <>
            <Keyboard size={16} aria-hidden="true" />
            Edit
          </>
        )}
      </button>
    </div>
  );
}

function formatShortcut(event: KeyboardEvent) {
  const key = normalizeShortcutKey(event.key);
  if (!key) {
    return "";
  }

  const parts = [
    event.ctrlKey && "Ctrl",
    event.altKey && "Alt",
    event.shiftKey && "Shift",
    event.metaKey && "Win",
    key
  ].filter(Boolean);

  return parts.length > 1 ? parts.join(" + ") : "";
}

function normalizeShortcutKey(key: string) {
  if (["Control", "Alt", "Shift", "Meta"].includes(key)) {
    return "";
  }
  if (key === " ") {
    return "Space";
  }
  if (key.length === 1) {
    return key.toUpperCase();
  }
  if (key.startsWith("Arrow")) {
    return key.replace("Arrow", "");
  }
  return key.length ? key[0].toUpperCase() + key.slice(1) : "";
}

function isUnsafeRecordedShortcut(shortcut: string) {
  const parts = shortcut.split("+").map((part) => part.trim().toLowerCase());
  const hasCtrl = parts.includes("ctrl");
  const hasAlt = parts.includes("alt");
  const hasShift = parts.includes("shift");
  const hasWin = parts.includes("win");
  const key = parts.find((part) => !["ctrl", "alt", "shift", "win"].includes(part));

  return Boolean(hasCtrl && !hasAlt && !hasShift && !hasWin && key?.length === 1 && /[a-z]/.test(key));
}

function ToggleRow({
  label,
  description,
  icon,
  checked,
  onChange
}: {
  label: string;
  description: string;
  icon: ReactNode;
  checked: boolean;
  onChange: (checked: boolean) => void;
}) {
  return (
    <label className="toggle-row">
      <span className="toggle-icon" aria-hidden="true">{icon}</span>
      <span className="toggle-copy">
        <strong>{label}</strong>
        <small>{description}</small>
      </span>
      <input type="checkbox" checked={checked} onChange={(event) => onChange(event.target.checked)} />
    </label>
  );
}

export default App;
