import { useEffect, useMemo, useState } from "react";
import clsx from "clsx";
import {
  BookOpen,
  ChevronDown,
  ClipboardCopy,
  Copy,
  Keyboard,
  Moon,
  RefreshCw,
  Replace,
  Search,
  Settings,
  Star,
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
import { promptCategories, promptTemplates, type PromptCategory } from "./data/prompts";

type ViewKey = "rewrite" | "library" | "favorites" | "settings";

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
      if (route === "favorites" || route === "settings" || route === "rewrite") {
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
    library: "Prompt Library",
    favorites: "Favorites",
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
            active={view === "library"}
            icon={BookOpen}
            label="Prompt Library"
            onClick={() => setView("library")}
          />
          <NavButton
            active={view === "favorites"}
            icon={Star}
            label="Favorites"
            onClick={() => setView("favorites")}
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
          <span>{settings.theme === "dark" ? "Dark" : "Light"}</span>
          <ChevronDown size={18} aria-hidden="true" />
        </button>
        <div className="sidebar-footer">
          <LogoMark small />
          <span>
            <strong>CorteX</strong>
            <small>AI Writing Assistant</small>
          </span>
        </div>
      </aside>

      <section className="workspace" aria-labelledby="workspace-title">
        <header className="workspace-header" data-tauri-drag-region>
          <div>
            <h1 id="workspace-title">{mainTitle}</h1>
            <p>{view === "rewrite" ? "Rewrite, improve, and perfect your text instantly." : viewSubtitle(view)}</p>
          </div>
          {view === "rewrite" && (
            <button className="ai-settings-button" type="button" onClick={() => setView("settings")}>
              <Settings size={20} aria-hidden="true" />
              <span>AI Settings</span>
              <ChevronDown size={18} aria-hidden="true" />
            </button>
          )}
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
        {view === "library" && <PromptLibrary onUse={(nextMode) => {
          setMode(nextMode);
          setView("rewrite");
          void runRewrite(nextMode);
        }} />}
        {view === "favorites" && <Favorites onUse={(nextMode) => {
          setMode(nextMode);
          setView("rewrite");
          void runRewrite(nextMode);
        }} />}
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
  if (view === "library") {
    return "Browse reusable prompts with variables for every writing workflow.";
  }
  if (view === "favorites") {
    return "Keep your most-used transformations one click away.";
  }
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
          <span>{input.length} characters</span>
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
          <span>{output.length} characters</span>
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

function PromptLibrary({ onUse }: { onUse: (mode: RewriteModeId) => void }) {
  const [query, setQuery] = useState("");
  const [category, setCategory] = useState<PromptCategory | "All">("All");
  const prompts = useMemo(
    () =>
      promptTemplates.filter((prompt) => {
        const matchesCategory = category === "All" || prompt.category === category;
        const matchesQuery =
          !query ||
          prompt.title.toLowerCase().includes(query.toLowerCase()) ||
          prompt.prompt.toLowerCase().includes(query.toLowerCase());
        return matchesCategory && matchesQuery;
      }),
    [category, query]
  );

  return (
    <div className="content-surface">
      <div className="library-toolbar">
        <label className="search-field">
          <Search size={20} aria-hidden="true" />
          <input
            value={query}
            onChange={(event) => setQuery(event.target.value)}
            placeholder="Search prompts"
          />
        </label>
        <div className="category-tabs" role="tablist" aria-label="Prompt categories">
          {(["All", ...promptCategories] as Array<PromptCategory | "All">).map((item) => (
            <button
              className={clsx(category === item && "active")}
              type="button"
              key={item}
              onClick={() => setCategory(item)}
            >
              {item}
            </button>
          ))}
        </div>
      </div>
      <div className="prompt-grid">
        {prompts.map((prompt) => (
          <article className="prompt-card" key={prompt.id}>
            <div>
              <span>{prompt.category}</span>
              <h2>{prompt.title}</h2>
              <p>{prompt.prompt}</p>
            </div>
            <div className="prompt-footer">
              <small>{prompt.variables.length ? prompt.variables.map((item) => `{${item}}`).join(" ") : "No variables"}</small>
              <button type="button" onClick={() => onUse(prompt.mode)}>
                Use
              </button>
            </div>
          </article>
        ))}
      </div>
    </div>
  );
}

function Favorites({ onUse }: { onUse: (mode: RewriteModeId) => void }) {
  const favorites = promptTemplates.filter((prompt) => prompt.favorite);
  return (
    <div className="content-surface favorite-list">
      {favorites.map((prompt) => (
        <article className="favorite-row" key={prompt.id}>
          <Star size={26} aria-hidden="true" />
          <div>
            <h2>{prompt.title}</h2>
            <p>{prompt.prompt}</p>
          </div>
          <button type="button" onClick={() => onUse(prompt.mode)}>
            Run
          </button>
        </article>
      ))}
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
    <div className="settings-grid">
      <section className="settings-panel">
        <h2>Provider</h2>
        <label>
          <span>AI provider</span>
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
          <span>Model</span>
          <input value={settings.provider.model} onChange={(event) => updateProvider({ model: event.target.value })} />
        </label>
        <label>
          <span>API key</span>
          <input
            value={settings.provider.apiKey ?? ""}
            type="password"
            placeholder="Stored locally"
            onChange={(event) => updateProvider({ apiKey: event.target.value })}
          />
        </label>
        <label>
          <span>Custom endpoint</span>
          <input
            value={settings.provider.endpoint ?? ""}
            placeholder="Optional"
            onChange={(event) => updateProvider({ endpoint: event.target.value })}
          />
        </label>
      </section>

      <section className="settings-panel">
        <h2>Desktop</h2>
        <label>
          <span>Floating window</span>
          <input value={settings.globalShortcut} onChange={(event) => update({ globalShortcut: event.target.value })} />
        </label>
        <label>
          <span>Grammar rewrite</span>
          <input value={settings.grammarShortcut} onChange={(event) => update({ grammarShortcut: event.target.value })} />
        </label>
        <label>
          <span>Professional rewrite</span>
          <input
            value={settings.professionalShortcut}
            onChange={(event) => update({ professionalShortcut: event.target.value })}
          />
        </label>
        <ToggleRow
          label="Auto copy after rewrite"
          checked={settings.autoCopy}
          onChange={(autoCopy) => update({ autoCopy })}
        />
        <ToggleRow
          label="Auto replace from shortcuts"
          checked={settings.autoReplace}
          onChange={(autoReplace) => update({ autoReplace })}
        />
      </section>

      <section className="settings-panel wide">
        <h2>Writing Defaults</h2>
        <div className="range-row">
          <label>
            <span>Temperature</span>
            <input
              type="range"
              min="0"
              max="1"
              step="0.05"
              value={settings.provider.temperature}
              onChange={(event) => updateProvider({ temperature: Number(event.target.value) })}
            />
          </label>
          <strong>{settings.provider.temperature.toFixed(2)}</strong>
        </div>
        <label>
          <span>Maximum tokens</span>
          <input
            type="number"
            min="100"
            max="4000"
            value={settings.provider.maxTokens}
            onChange={(event) => updateProvider({ maxTokens: Number(event.target.value) })}
          />
        </label>
        <label>
          <span>Default language</span>
          <input value={settings.defaultLanguage} onChange={(event) => update({ defaultLanguage: event.target.value })} />
        </label>
      </section>
    </div>
  );
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
