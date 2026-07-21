import { useEffect, useState } from "react";
import clsx from "clsx";
import {
  Check,
  CheckCircle2,
  Clock3,
  Copy,
  Minus,
  Replace,
  X,
  Zap
} from "lucide-react";
import {
  copyText,
  getPopupPayload,
  getSettings,
  hideCurrentWindow,
  listenToPopupPayload,
  listenToSettingsUpdates,
  replaceSelectedText,
  rewriteText,
  setWindowTheme,
  startCurrentWindowDrag,
  type PopupPayload
} from "./lib/desktop";
import { defaultInput, defaultOutput, modeLabel, rewriteModes, type RewriteModeId } from "./data/modes";

const initialPayload: PopupPayload = {
  input: defaultInput,
  output: defaultOutput,
  mode: "fixGrammar",
  provider: "offline",
  usedOfflineFallback: true,
  characterCount: defaultOutput.length,
  elapsedMs: 0,
  source: "manual",
  loading: false
};

function formatElapsed(milliseconds: number) {
  if (!Number.isFinite(milliseconds) || milliseconds <= 0) {
    return "0ms";
  }
  if (milliseconds < 1000) {
    return `${Math.round(milliseconds)}ms`;
  }
  return `${(milliseconds / 1000).toFixed(2)}s`;
}

function PopupApp() {
  const [payload, setPayload] = useState(initialPayload);
  const [mode, setMode] = useState<RewriteModeId>("fixGrammar");
  const [busy, setBusy] = useState(false);
  const [panelTheme, setPanelTheme] = useState<"system" | "dark" | "light">("dark");
  const [systemPrefersDark, setSystemPrefersDark] = useState(() =>
    window.matchMedia("(prefers-color-scheme: dark)").matches
  );
  const visibleCharacterCount = Array.from(payload.output || "").length;
  const characterLimit = Math.max(1000, visibleCharacterCount);

  useEffect(() => {
    let disposed = false;
    let receivedPayloadEvent = false;
    let stopPayloadListener: (() => void) | undefined;
    let stopSettingsListener: (() => void) | undefined;

    const applySettings = (settings: { popupTheme?: string }) => {
      if (disposed) return;
      setPanelTheme(
        settings.popupTheme === "light" || settings.popupTheme === "system" ? settings.popupTheme : "dark"
      );
    };

    void getSettings().then(applySettings).catch(() => undefined);
    void (async () => {
      // Subscribe first. Otherwise a fast provider response can arrive between the
      // initial state request and listener registration, leaving the panel blank.
      stopPayloadListener = await listenToPopupPayload((value) => {
        if (disposed) return;
        receivedPayloadEvent = true;
        setPayload(value);
        setMode(value.mode);
        setBusy(Boolean(value.loading));
      });
      const value = await getPopupPayload();
      if (!disposed && value && !receivedPayloadEvent) {
        setPayload(value);
        setMode(value.mode);
        setBusy(Boolean(value.loading));
      }
    })().catch(() => undefined);

    void listenToSettingsUpdates(applySettings)
      .then((unlisten) => {
        stopSettingsListener = unlisten;
      })
      .catch(() => undefined);

    return () => {
      disposed = true;
      stopPayloadListener?.();
      stopSettingsListener?.();
    };
  }, []);

  useEffect(() => {
    const colorScheme = window.matchMedia("(prefers-color-scheme: dark)");
    const syncSystemTheme = () => setSystemPrefersDark(colorScheme.matches);
    colorScheme.addEventListener("change", syncSystemTheme);
    return () => colorScheme.removeEventListener("change", syncSystemTheme);
  }, []);

  const activePanelTheme = panelTheme === "system" ? (systemPrefersDark ? "dark" : "light") : panelTheme;

  useEffect(() => {
    document.documentElement.style.colorScheme = activePanelTheme;
    void setWindowTheme(activePanelTheme).catch(() => undefined);
  }, [activePanelTheme]);

  useEffect(() => {
    function onKeyDown(event: KeyboardEvent) {
      if (event.key === "Escape") {
        event.preventDefault();
        event.stopPropagation();
        void hideCurrentWindow();
        return;
      }
      if (event.key === "Enter" && event.ctrlKey) {
        event.preventDefault();
        void handleReplace();
      }
      if (event.key === "Enter" && !event.shiftKey && !event.ctrlKey) {
        event.preventDefault();
        void runRewrite(mode);
      }
    }

    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [mode, payload.input]);

  async function runRewrite(nextMode: RewriteModeId) {
    const source = payload.input.trim() || payload.output.trim();
    if (!source) {
      return;
    }
    setBusy(true);
    setMode(nextMode);
    try {
      const response = await rewriteText({ input: source, mode: nextMode });
      setPayload({ ...response, source: "manual" });
    } catch (error) {
      const message = error instanceof Error ? error.message : "Rewrite failed. Please try again.";
      setPayload((current) => ({
        ...current,
        output: message,
        characterCount: Array.from(message).length,
        loading: false
      }));
    } finally {
      setBusy(false);
    }
  }

  async function handleCopy() {
    await copyText(payload.output);
  }

  async function handleReplace() {
    await replaceSelectedText(payload.output);
    await hideCurrentWindow();
  }

  function handleTitlebarPointerDown(event: React.PointerEvent<HTMLElement>) {
    if (event.button !== 0 || (event.target as HTMLElement).closest("button, kbd")) {
      return;
    }

    event.preventDefault();
    void startCurrentWindowDrag();
  }

  return (
    <main className="popup-root" data-theme={activePanelTheme} aria-label="CorteX floating rewrite popup">
      <section className="popup-card">
        <header className="popup-titlebar" onPointerDownCapture={handleTitlebarPointerDown}>
          <div className="popup-brand">
            <img src={`${import.meta.env.BASE_URL}cortex-icon.png`} alt="" aria-hidden="true" />
            <strong>CorteX</strong>
          </div>
          <div className="popup-window-actions">
            <button type="button" aria-label="Hide popup" onClick={() => void hideCurrentWindow()}>
              <Minus size={19} aria-hidden="true" />
            </button>
            <kbd>ESC</kbd>
            <button type="button" aria-label="Close popup" onClick={() => void hideCurrentWindow()}>
              <X size={20} aria-hidden="true" />
            </button>
          </div>
        </header>

        <nav className="popup-mode-tabs" aria-label="Rewrite mode">
          {rewriteModes.slice(0, 6).map((item) => (
            <button
              className={clsx(mode === item.id && "active")}
              type="button"
              key={item.id}
              onClick={() => runRewrite(item.id)}
              aria-pressed={mode === item.id}
            >
              {mode === item.id ? <Check size={17} aria-hidden="true" /> : <item.icon size={18} aria-hidden="true" />}
              <span>{item.label}</span>
            </button>
          ))}
        </nav>

        <section className="popup-output" aria-labelledby="popup-output-label">
          <div className="popup-output-heading">
            <h1 id="popup-output-label">Rewritten Text</h1>
          </div>
          <div className={clsx("popup-output-box", busy && "rewriting")} role="textbox" aria-readonly="true" tabIndex={0}>
            <div className="popup-output-copy" aria-live="polite">
              {busy ? "Rewriting..." : payload.output || `Ready to ${modeLabel(mode).toLowerCase()}. Select text and use Ctrl + Alt + X.`}
            </div>
            <span className="popup-character-count">
              {visibleCharacterCount.toLocaleString()} / {characterLimit.toLocaleString()}
            </span>
          </div>
        </section>

        <footer className="popup-footer">
          <div className="popup-status" aria-live="polite">
            <span className="popup-ready"><CheckCircle2 size={18} aria-hidden="true" />{busy ? "Rewriting" : "Ready"}</span>
            <span className="popup-elapsed"><Clock3 size={17} aria-hidden="true" />{busy ? "..." : formatElapsed(payload.elapsedMs)}</span>
          </div>
          <div className="popup-actions">
            <button type="button" onClick={handleReplace} className="ghost-action">
              <Replace size={20} aria-hidden="true" />
              Replace
            </button>
            <button type="button" onClick={handleCopy} className="ghost-action">
              <Copy size={20} aria-hidden="true" />
              Copy
            </button>
            <button type="button" onClick={() => runRewrite(mode)} className="popup-primary" disabled={busy}>
              <Zap size={21} aria-hidden="true" />
              {busy ? "Writing" : "Rewrite"}
            </button>
          </div>
        </footer>
      </section>
    </main>
  );
}

export default PopupApp;
