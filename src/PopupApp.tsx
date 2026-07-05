import { useEffect, useState } from "react";
import clsx from "clsx";
import {
  Check,
  ClipboardCopy,
  Copy,
  CornerDownLeft,
  Replace,
  Sparkles,
  X,
  Zap
} from "lucide-react";
import {
  copyText,
  getPopupPayload,
  hideCurrentWindow,
  listenToPopupPayload,
  replaceSelectedText,
  rewriteText,
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
  source: "manual"
};

function PopupApp() {
  const [payload, setPayload] = useState(initialPayload);
  const [mode, setMode] = useState<RewriteModeId>("fixGrammar");
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    getPopupPayload().then((value) => {
      if (value) {
        setPayload(value);
        setMode(value.mode);
      }
    });

    listenToPopupPayload((value) => {
      setPayload(value);
      setMode(value.mode);
    }).then((unlisten) => () => unlisten());
  }, []);

  useEffect(() => {
    function onKeyDown(event: KeyboardEvent) {
      if (event.key === "Escape") {
        void hideCurrentWindow();
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

  return (
    <main className="popup-root" aria-label="CorteX floating rewrite popup">
      <section className="popup-card">
        <header className="popup-titlebar" data-tauri-drag-region>
          <div className="popup-brand">
            <Sparkles size={21} aria-hidden="true" />
            <strong>CorteX</strong>
            <span>{payload.provider === "offline" ? "Offline" : payload.provider}</span>
          </div>
          <div className="popup-window-actions">
            <kbd>ESC</kbd>
            <button type="button" aria-label="Close popup" onClick={() => hideCurrentWindow()}>
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
            <h1 id="popup-output-label">Output Text</h1>
            <span>{payload.output.length} chars</span>
          </div>
          <div className="popup-output-box" role="textbox" aria-readonly="true" tabIndex={0}>
            {payload.output || `Ready to ${modeLabel(mode).toLowerCase()}. Select text and use Ctrl + Alt + Z.`}
          </div>
        </section>

        <footer className="popup-footer">
          <div className="popup-shortcuts" aria-hidden="true">
            <span>
              <kbd>
                <CornerDownLeft size={16} />
              </kbd>
              Run
            </span>
            <span>
              <kbd>Ctrl + Enter</kbd>
              Replace
            </span>
            <span>
              <kbd>Ctrl + C</kbd>
              Copy
            </span>
          </div>
          <div className="popup-actions">
            <button type="button" onClick={handleCopy} className="ghost-action">
              <ClipboardCopy size={20} aria-hidden="true" />
              Copy
            </button>
            <button type="button" onClick={handleReplace} className="ghost-action">
              <Replace size={20} aria-hidden="true" />
              Replace
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
