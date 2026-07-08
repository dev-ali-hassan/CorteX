import { useEffect, useRef, useState } from "react";
import type { ChangeEvent, KeyboardEvent } from "react";
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
  RefreshCw,
  Settings,
  Sparkles,
  Trash2,
  Wand2,
  X,
  Minus,
  Square
} from "lucide-react";
import {
  copyText,
  defaultSettings,
  getSettings,
  listenToTrayNavigation,
  replaceSelectedText,
  rewriteText,
  saveSettings,
  windowAction,
  type AppSettings
} from "./lib/desktop";
import { extractTextFromDocument } from "./lib/documentImport";
import { defaultInput, defaultOutput, rewriteModes, type RewriteModeId } from "./data/modes";

type ViewKey = "rewrite" | "settings";

const providerLabels = {
  offline: "Offline utilities",
  openai: "OpenAI",
  openrouter: "OpenRouter",
  gemini: "Google Gemini",
  anthropic: "Anthropic Claude",
  ollama: "Ollama local"
};

const providerModels = {
  offline: "local-cleanup",
  openai: "gpt-4o-mini",
  openrouter: "openai/gpt-4o-mini",
  gemini: "gemini-1.5-flash",
  anthropic: "claude-3-5-haiku-latest",
  ollama: "llama3.1"
};

function App() {
  const [view, setView] = useState<ViewKey>("rewrite");
  const [input, setInput] = useState(defaultInput);
  const [output, setOutput] = useState(defaultOutput);
  const [mode, setMode] = useState<RewriteModeId>("fixGrammar");
  const [settings, setSettings] = useState<AppSettings>(defaultSettings);
  const [status, setStatus] = useState("Ready");
  const [loading, setLoading] = useState(false);
  const [copied, setCopied] = useState(false);
  const [settingsJumpTarget, setSettingsJumpTarget] = useState<string | null>(null);

  useEffect(() => {
    getSettings()
      .then((value) => setSettings(value))
      .catch(() => setSettings(defaultSettings));

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
      const scroller = document.querySelector<HTMLElement>(".settings-shell");

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

  async function runRewrite(nextMode = mode) {
    const text = input.trim();
    if (!text) {
      setStatus("Add text first");
      return;
    }

    setLoading(true);
    setCopied(false);
    setStatus("Rewriting");
    try {
      const response = await rewriteText({
        input: text,
        mode: nextMode,
        targetLanguage: settings.defaultLanguage
      });
      setOutput(response.output);
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
  const hasConnectedProvider =
    Boolean(settings.provider.apiKey?.trim()) ||
    settings.provider.provider === "ollama";
  const activeTheme = settings.theme === "system" ? "dark" : settings.theme;

  function openShortcutSettings() {
    setSettingsJumpTarget("shortcuts-section");
    setView("settings");
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
        <div className="model-card">
          <span className="model-icon" aria-hidden="true">
            <Sparkles size={22} />
          </span>
          <span>
            <small>AI Model</small>
            <strong>{settings.provider.model || "GPT-4o"}</strong>
            <em className={clsx(!hasConnectedProvider && "not-connected")}>
              {hasConnectedProvider ? "Connected" : "Not connected"}
            </em>
          </span>
        </div>
        <div className="sidebar-version">
          <span>v1.0.0</span>
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
            status={status}
            copied={copied}
            setInput={setInput}
            setMode={(nextMode) => {
              setMode(nextMode);
              void runRewrite(nextMode);
            }}
            onRewrite={() => runRewrite()}
            onCopy={handleCopy}
            onReplace={handleReplace}
            onDocumentImport={handleDocumentImport}
          />
        )}
        {view === "settings" && (
          <SettingsView
            settings={settings}
            onChange={async (nextSettings) => {
              setSettings(nextSettings);
              const saved = await saveSettings(nextSettings);
              setSettings(saved);
              setStatus("Settings saved");
            }}
          />
        )}
      </section>
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

function QuickRewrite({
  input,
  output,
  mode,
  loading,
  status,
  copied,
  setInput,
  setMode,
  onRewrite,
  onCopy,
  onReplace,
  onDocumentImport
}: {
  input: string;
  output: string;
  mode: RewriteModeId;
  loading: boolean;
  status: string;
  copied: boolean;
  setInput: (value: string) => void;
  setMode: (value: RewriteModeId) => void;
  onRewrite: () => void;
  onCopy: () => void;
  onReplace: () => void;
  onDocumentImport: (file: File) => Promise<void>;
}) {
  const selectedMode = rewriteModes.find((item) => item.id === mode);
  const inputWords = countWords(input);
  const outputWords = countWords(output);
  const fileInputRef = useRef<HTMLInputElement>(null);

  async function handleFileChange(event: ChangeEvent<HTMLInputElement>) {
    const file = event.target.files?.[0];
    if (!file) {
      return;
    }
    await onDocumentImport(file);
    event.target.value = "";
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
          <span className="word-count">{inputWords} words</span>
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
        <h2 id="mode-label">Rewrite mode</h2>
        <div className="mode-grid">
          {rewriteModes.map((item) => (
            <button
              className={clsx("mode-tile", mode === item.id && "selected")}
              type="button"
              key={item.id}
              onClick={() => setMode(item.id)}
              aria-pressed={mode === item.id}
              title={item.description}
            >
              <item.icon size={34} strokeWidth={2} aria-hidden="true" />
              <span>{item.label}</span>
            </button>
          ))}
        </div>
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
            {selectedMode?.label ?? "Rewrite"} ready
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
            <span>1.2s</span>
            <span className="word-count">{outputWords} words</span>
          </div>
          <div className="output-actions">
            <button className={clsx("primary-action", copied && "copied")} type="button" onClick={onCopy}>
              {copied ? <CheckCircle2 size={20} aria-hidden="true" /> : <Copy size={23} aria-hidden="true" />}
              <span>{copied ? "Copied" : "Copy"}</span>
            </button>
          </div>
        </div>
      </section>

      <footer className="action-row">
        <button className="secondary-action" type="button" onClick={onRewrite} disabled={loading}>
          <RefreshCw size={21} aria-hidden="true" />
          <span>{loading ? "Rewriting" : "Rewrite Again"}</span>
          <kbd>Enter</kbd>
        </button>
      </footer>
    </div>
  );
}

function SettingsView({
  settings,
  onChange
}: {
  settings: AppSettings;
  onChange: (settings: AppSettings) => void;
}) {
  const providerEntries = Object.entries(providerLabels) as Array<[AppSettings["provider"]["provider"], string]>;

  function update(next: Partial<AppSettings>) {
    onChange({ ...settings, ...next });
  }

  function updateProvider(next: Partial<AppSettings["provider"]>) {
    const provider = next.provider ?? settings.provider.provider;
    onChange({
      ...settings,
      provider: {
        ...settings.provider,
        model: next.provider ? providerModels[provider] : settings.provider.model,
        ...next
      }
    });
  }

  return (
    <div className="settings-shell">
      <section className="settings-panel settings-stack">
        <div className="settings-section" id="shortcuts-section">
          <h2>AI</h2>
          <label>
            <span>Provider</span>
            <select
              value={settings.provider.provider}
              onChange={(event) => updateProvider({ provider: event.target.value as AppSettings["provider"]["provider"] })}
            >
              {providerEntries.map(([value, label]) => (
                <option value={value} key={value}>
                  {label}
                </option>
              ))}
            </select>
          </label>
          <label>
            <span>API Key</span>
            <input value={settings.provider.apiKey?.trim() ? "************" : "Not connected"} readOnly />
          </label>
          <button className="test-connection-button" type="button">
            Test Connection
          </button>
        </div>

        <div className="settings-divider" />

        <div className="settings-section">
          <h2>General</h2>
          <label>
            <span>Custom prompt</span>
            <textarea
              className="settings-textarea"
              value={settings.customPrompt}
              placeholder="Tell CorteX how to rewrite when an AI provider is connected."
              onChange={(event) => update({ customPrompt: event.target.value })}
            />
          </label>
          <ToggleRow
            label="Launch at startup"
            checked={settings.launchAtStartup}
            onChange={(launchAtStartup) => update({ launchAtStartup })}
          />
          <ToggleRow
            label="Auto-copy result"
            checked={settings.autoCopy}
            onChange={(autoCopy) => update({ autoCopy })}
          />
          <ToggleRow
            label="Auto-replace selection"
            checked={settings.autoReplace}
            onChange={(autoReplace) => update({ autoReplace })}
          />
        </div>

        <div className="settings-divider" />

        <div className="settings-section">
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

        <div className="settings-divider" />

        <div className="settings-section">
          <h2>Appearance</h2>
          <div className="theme-options" role="radiogroup" aria-label="Theme">
            <span>Theme</span>
            {[
              ["dark", "Dark"],
              ["light", "Light"],
              ["purple", "Purple"]
            ].map(([value, label]) => (
              <label key={value}>
                <input
                  type="radio"
                  name="theme"
                  value={value}
                  checked={settings.theme === value}
                  onChange={() => update({ theme: value as AppSettings["theme"] })}
                />
                <span>{label}</span>
              </label>
            ))}
          </div>
        </div>

        <div className="settings-divider" />

        <div className="settings-section privacy-section">
          <h2>Privacy</h2>
          <p>✓ Stored locally</p>
          <p>✓ No analytics</p>
          <p>✓ No cloud storage</p>
        </div>
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

  function handleKeyDown(event: KeyboardEvent<HTMLButtonElement>) {
    if (!recording) {
      return;
    }

    event.preventDefault();
    event.stopPropagation();

    const shortcut = formatShortcut(event);
    if (shortcut) {
      onChange(shortcut);
      setRecording(false);
    }
  }

  return (
    <div className="shortcut-recorder">
      <div>
        <span>{label}</span>
        <small>Current: {value}</small>
      </div>
      <button
        type="button"
        className={clsx(recording && "recording")}
        onClick={() => setRecording(true)}
        onKeyDown={handleKeyDown}
        onBlur={() => setRecording(false)}
      >
        {recording ? "Press shortcut" : "Change"}
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

function ToggleRow({
  label,
  checked,
  onChange
}: {
  label: string;
  checked: boolean;
  onChange: (checked: boolean) => void;
}) {
  return (
    <label className="toggle-row">
      <span>{label}</span>
      <input type="checkbox" checked={checked} onChange={(event) => onChange(event.target.checked)} />
    </label>
  );
}

export default App;
