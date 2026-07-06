import { useEffect, useState } from "react";
import type { KeyboardEvent } from "react";
import clsx from "clsx";
import {
  ChevronDown,
  ClipboardCopy,
  Copy,
  Keyboard,
  Moon,
  RefreshCw,
  Replace,
  Settings,
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

  async function runRewrite(nextMode = mode) {
    const text = input.trim();
    if (!text) {
      setStatus("Add text first");
      return;
    }

    setLoading(true);
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
    setStatus("Copied");
  }

  async function handleReplace() {
    if (!output.trim()) {
      return;
    }
    await replaceSelectedText(output);
    setStatus("Replaced selected text");
  }

  const mainTitle = {
    rewrite: "Quick Rewrite",
    settings: "Settings"
  }[view];

  return (
    <main className="desktop-window" aria-label="CorteX desktop app">
      <TitleControls />
      <aside className="sidebar" data-tauri-drag-region>
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
        <div className="shortcut-card">
          <span className="shortcut-icon" aria-hidden="true">
            <Keyboard size={26} />
          </span>
          <span>
            <strong>{settings.globalShortcut}</strong>
            <small>Global Shortcut</small>
          </span>
        </div>
        <button className="theme-button" type="button" aria-label="Theme menu">
          <Moon size={24} aria-hidden="true" />
          <span>{settings.theme === "system" ? "System" : settings.theme === "dark" ? "Dark" : "Light"}</span>
          <ChevronDown size={18} aria-hidden="true" />
        </button>
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
            setInput={setInput}
            setMode={(nextMode) => {
              setMode(nextMode);
              void runRewrite(nextMode);
            }}
            onRewrite={() => runRewrite()}
            onCopy={handleCopy}
            onReplace={handleReplace}
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
  return "Choose providers, shortcuts, and behavior for the desktop assistant.";
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

function QuickRewrite({
  input,
  output,
  mode,
  loading,
  status,
  setInput,
  setMode,
  onRewrite,
  onCopy,
  onReplace
}: {
  input: string;
  output: string;
  mode: RewriteModeId;
  loading: boolean;
  status: string;
  setInput: (value: string) => void;
  setMode: (value: RewriteModeId) => void;
  onRewrite: () => void;
  onCopy: () => void;
  onReplace: () => void;
}) {
  return (
    <div className="rewrite-surface">
      <section className="text-panel" aria-labelledby="input-label">
        <div className="panel-heading">
          <label id="input-label" htmlFor="input-text">
            Input
          </label>
        </div>
        <textarea
          id="input-text"
          value={input}
          onChange={(event) => setInput(event.target.value)}
          spellCheck
          aria-describedby="rewrite-status"
        />
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
        <div className="panel-heading">
          <label id="output-label" htmlFor="output-text">
            Output
          </label>
        </div>
        <textarea id="output-text" value={output} readOnly />
        <button className="floating-copy" type="button" onClick={onCopy} aria-label="Copy output">
          <ClipboardCopy size={22} aria-hidden="true" />
        </button>
      </section>

      <footer className="action-row">
        <button className="secondary-action" type="button" onClick={onRewrite} disabled={loading}>
          <RefreshCw size={21} aria-hidden="true" />
          <span>{loading ? "Rewriting" : "Rewrite"}</span>
          <kbd>Enter</kbd>
        </button>
        <p id="rewrite-status" role="status">
          {status}
        </p>
        <button className="secondary-action" type="button" onClick={onReplace}>
          <Replace size={21} aria-hidden="true" />
          <span>Replace</span>
          <kbd>Ctrl + Enter</kbd>
        </button>
        <button className="primary-action" type="button" onClick={onCopy}>
          <Copy size={23} aria-hidden="true" />
          <span>Copy</span>
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
        <div className="settings-section">
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
        </div>

        <div className="settings-divider" />

        <div className="settings-section">
          <h2>Appearance</h2>
          <div className="theme-options" role="radiogroup" aria-label="Theme">
            <span>Theme</span>
            {[
              ["dark", "Dark"],
              ["system", "System"]
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
