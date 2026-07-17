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
  hideCurrentWindow,
  listenToPopupPayload,
  replaceSelectedText,
  rewriteText,
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
  source: "manual"
};

function formatElapsed(milliseconds: number) {
  if (milliseconds < 1000) {
    return `${milliseconds}ms`;
  }
  return `${(milliseconds / 1000).toFixed(2)}s`;
}

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

  function handleTitlebarMouseDown(event: React.MouseEvent<HTMLElement>) {
    if (event.button !== 0 || (event.target as HTMLElement).closest("button, kbd")) {
      return;
    }
    void startCurrentWindowDrag();
  }

  return (
    <main className="popup-root" aria-label="CorteX floating rewrite popup">
      <section className="popup-card">
        <header className="popup-titlebar" data-tauri-drag-region onMouseDown={handleTitlebarMouseDown}>
          <div className="popup-brand" data-tauri-drag-region>
            <img src="/cortex-icon.png" alt="" aria-hidden="true" />
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
              {busy ? "Rewriting..." : payload.output || `Ready to ${modeLabel(mode).toLowerCase()}. Select text and use Ctrl + Alt + Z.`}
            </div>
            <span className="popup-character-count">{payload.characterCount} characters</span>
          </div>
        </section>

        <footer className="popup-footer">
          <div className="popup-status" aria-live="polite">
            <span className="popup-ready"><CheckCircle2 size={18} aria-hidden="true" />Ready</span>
            <span className="popup-elapsed"><Clock3 size={17} aria-hidden="true" />{formatElapsed(payload.elapsedMs)}</span>
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
