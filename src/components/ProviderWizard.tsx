import { useEffect, useMemo, useState } from "react";
import {
  ArrowLeft,
  ArrowRight,
  Bot,
  BrainCircuit,
  Check,
  CheckCircle2,
  Cloud,
  ExternalLink,
  Eye,
  EyeOff,
  LoaderCircle,
  LockKeyhole,
  Network,
  Orbit,
  Route,
  Server,
  Sparkles,
  Waves,
  Wind,
  X,
  Zap
} from "lucide-react";
import clsx from "clsx";
import {
  getProviderDefinition,
  providerDefinitions,
  type ConnectableProviderId,
  type ProviderDefinition
} from "../data/providers";
import { openProviderGuide, testProviderConnection, type ProviderSettings } from "../lib/desktop";

type WizardStep = 1 | 2 | 3 | 4 | 5;

type ProviderWizardProps = {
  initialSettings: ProviderSettings;
  onClose: () => void;
  onConnected: (settings: ProviderSettings) => Promise<void>;
};

function ProviderMark({ provider, size = 22 }: { provider: ConnectableProviderId; size?: number }) {
  const Icon = {
    gemini: Sparkles,
    groq: Zap,
    openrouter: Route,
    openai: Bot,
    anthropic: BrainCircuit,
    mistral: Wind,
    cohere: Network,
    xai: Orbit,
    deepseek: Waves,
    ollama: Server
  }[provider];
  return <Icon size={size} aria-hidden="true" />;
}

function candidateSettings(initial: ProviderSettings, definition: ProviderDefinition, apiKey: string): ProviderSettings {
  return {
    ...initial,
    provider: definition.id,
    model: definition.model,
    apiKey: definition.keyRequired ? apiKey.trim() : undefined,
    endpoint: undefined
  };
}

export function ProviderWizard({ initialSettings, onClose, onConnected }: ProviderWizardProps) {
  const initialProvider = initialSettings.provider === "offline" ? "gemini" : initialSettings.provider;
  const [step, setStep] = useState<WizardStep>(1);
  const [selectedId, setSelectedId] = useState<ConnectableProviderId>(initialProvider);
  const [apiKey, setApiKey] = useState(initialSettings.apiKey ?? "");
  const [showKey, setShowKey] = useState(false);
  const [testPhase, setTestPhase] = useState(0);
  const [testError, setTestError] = useState("");
  const provider = useMemo(() => getProviderDefinition(selectedId), [selectedId]);

  useEffect(() => {
    const handleKeyDown = (event: globalThis.KeyboardEvent) => {
      if (event.key === "Escape") onClose();
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [onClose]);

  useEffect(() => {
    if (step !== 4) return;
    let active = true;
    setTestError("");
    setTestPhase(0);
    const progress = window.setInterval(() => setTestPhase((value) => Math.min(value + 1, 3)), 420);

    void testProviderConnection(candidateSettings(initialSettings, provider, apiKey))
      .then(async () => {
        if (!active) return;
        setTestPhase(4);
        await onConnected(candidateSettings(initialSettings, provider, apiKey));
        if (active) window.setTimeout(() => active && setStep(5), 350);
      })
      .catch((error) => {
        if (!active) return;
        setTestError(error instanceof Error ? error.message : "CorteX could not verify this provider.");
      })
      .finally(() => window.clearInterval(progress));

    return () => {
      active = false;
      window.clearInterval(progress);
    };
  }, [step]);

  function selectProvider(id: ConnectableProviderId) {
    setSelectedId(id);
    setApiKey(id === initialSettings.provider ? initialSettings.apiKey ?? "" : "");
  }

  return (
    <div className="provider-wizard-backdrop" role="presentation" onMouseDown={(event) => event.target === event.currentTarget && onClose()}>
      <section className="provider-wizard" role="dialog" aria-modal="true" aria-labelledby="provider-wizard-title">
        <header className="provider-wizard-header">
          <div className="provider-wizard-heading">
            {step > 1 && <span className="provider-wizard-mark"><ProviderMark provider={selectedId} size={28} /></span>}
            <div>
              <h2 id="provider-wizard-title">
                {step === 1 && "Connect an AI Provider"}
                {step === 2 && provider.name}
                {step === 3 && (provider.keyRequired ? `Enter Your ${provider.shortName} API Key` : `Connect ${provider.shortName}`)}
                {step === 4 && "Testing Connection"}
                {step === 5 && "Connected Successfully"}
              </h2>
              <p>
                {step === 1 && "Choose the provider you want CorteX to use for rewriting."}
                {step === 2 && provider.description}
                {step === 3 && (provider.keyRequired ? `Paste the API key you copied from ${provider.guideLabel}.` : "CorteX will connect to the provider running on this computer.")}
                {step === 4 && `Verifying your ${provider.name} connection.`}
                {step === 5 && `${provider.name} is now active in CorteX.`}
              </p>
            </div>
          </div>
          <button className="provider-wizard-close" type="button" aria-label="Close provider setup" onClick={onClose}>
            <X size={19} />
          </button>
        </header>

        <div className="provider-wizard-progress" aria-label={`Step ${step} of 5`}>
          {[1, 2, 3, 4, 5].map((item) => <span key={item} className={clsx(item <= step && "active")} />)}
        </div>

        <div className="provider-wizard-body">
          {step === 1 && (
            <div className="provider-picker">
              {(["recommended", "premium", "local"] as const).map((group) => (
                <section key={group} className="provider-picker-group">
                  <h3>{group === "recommended" ? "Recommended" : group === "premium" ? "Premium Providers" : "Local Provider"}</h3>
                  <div className="provider-picker-list">
                    {providerDefinitions.filter((item) => item.group === group).map((item) => (
                      <button
                        type="button"
                        className={clsx("provider-picker-row", item.id === selectedId && "selected")}
                        key={item.id}
                        onClick={() => selectProvider(item.id)}
                        onDoubleClick={() => {
                          selectProvider(item.id);
                          setStep(2);
                        }}
                      >
                        <span className={`provider-logo provider-${item.id}`}><ProviderMark provider={item.id} /></span>
                        <span className="provider-picker-copy">
                          <strong>{item.name}</strong>
                          <small>{item.description}</small>
                        </span>
                        {item.badge && <em>{item.badge}</em>}
                        <ChevronArrow selected={item.id === selectedId} />
                      </button>
                    ))}
                  </div>
                </section>
              ))}
            </div>
          )}

          {step === 2 && (
            <div className="provider-guide">
              <div className="provider-benefits">
                <span><Check size={16} /> Guided setup</span>
                <span><Check size={16} /> Verified connection</span>
                <span><Check size={16} /> Ready for rewriting</span>
              </div>
              <div className="provider-guide-card">
                <div>
                  <h3>{provider.keyRequired ? "How to get your API key" : "How to prepare Ollama"}</h3>
                  <ol>
                    {provider.setupSteps.map((item) => <li key={item}><span>{provider.setupSteps.indexOf(item) + 1}</span>{item}</li>)}
                  </ol>
                </div>
                <button type="button" className="provider-guide-link" onClick={() => void openProviderGuide(provider.guideUrl)}>
                  <ProviderMark provider={selectedId} size={26} />
                  <strong>{provider.guideLabel}</strong>
                  <span>Open guide <ExternalLink size={14} /></span>
                </button>
              </div>
            </div>
          )}

          {step === 3 && (
            <div className="provider-key-step">
              {provider.keyRequired ? (
                <label>
                  <span>API Key</span>
                  <span className="provider-key-input">
                    <input
                      type={showKey ? "text" : "password"}
                      value={apiKey}
                      onChange={(event) => setApiKey(event.target.value)}
                      placeholder={`Paste your ${provider.shortName} API key`}
                      autoComplete="off"
                      autoFocus
                    />
                    <button type="button" aria-label={showKey ? "Hide API key" : "Show API key"} onClick={() => setShowKey((value) => !value)}>
                      {showKey ? <EyeOff size={18} /> : <Eye size={18} />}
                    </button>
                  </span>
                </label>
              ) : (
                <div className="provider-local-check">
                  <Server size={34} />
                  <div><strong>Ollama must be running</strong><p>CorteX will verify the local service and the configured model.</p></div>
                </div>
              )}
              <div className="provider-security-note">
                <LockKeyhole size={21} />
                <div><strong>Stored on your device</strong><p>Your credential is saved locally and sent only to the selected provider.</p></div>
              </div>
            </div>
          )}

          {step === 4 && (
            <div className="provider-testing">
              <span className={clsx("provider-testing-orb", testError && "failed")}>
                {testError ? <X size={38} /> : <LoaderCircle size={42} />}
              </span>
              <h3>{testError ? "Connection could not be verified" : "Testing connection..."}</h3>
              {!testError && (
                <div className="provider-test-checks">
                  {["Checking provider settings", "Contacting provider", "Validating credentials", "Finalizing connection"].map((item, index) => (
                    <span key={item} className={clsx(index < testPhase && "complete", index === testPhase && "current")}>
                      {index < testPhase ? <CheckCircle2 size={17} /> : <LoaderCircle size={17} />}{item}
                    </span>
                  ))}
                </div>
              )}
              {testError && <p className="provider-test-error">{testError}</p>}
            </div>
          )}

          {step === 5 && (
            <div className="provider-success">
              <span className="provider-success-icon"><Check size={48} /></span>
              <h3>Connected Successfully</h3>
              <p>{provider.name} is ready to rewrite, improve, summarize, and simplify your text.</p>
              <div className="provider-success-list">
                <span><Check size={16} /> Rewrite and improve text</span>
                <span><Check size={16} /> Use all AI rewrite modes</span>
                <span><Check size={16} /> Run custom prompts</span>
              </div>
            </div>
          )}
        </div>

        <footer className="provider-wizard-footer">
          {step === 1 && <button className="secondary" type="button" onClick={onClose}>Skip for now</button>}
          {step > 1 && step < 4 && <button className="secondary" type="button" onClick={() => setStep((step - 1) as WizardStep)}><ArrowLeft size={16} /> Back</button>}
          {step === 4 && testError && <button className="secondary" type="button" onClick={() => setStep(3)}><ArrowLeft size={16} /> Back</button>}
          <span />
          {step === 1 && <button className="primary" type="button" onClick={() => setStep(2)}>Continue <ArrowRight size={16} /></button>}
          {step === 2 && <button className="primary" type="button" onClick={() => setStep(3)}>Continue <ArrowRight size={16} /></button>}
          {step === 3 && <button className="primary" type="button" disabled={provider.keyRequired && !apiKey.trim()} onClick={() => setStep(4)}>Test & Connect <ArrowRight size={16} /></button>}
          {step === 4 && testError && <button className="primary" type="button" onClick={() => { setTestError(""); setStep(3); window.setTimeout(() => setStep(4), 0); }}>Try Again</button>}
          {step === 5 && <button className="primary" type="button" onClick={onClose}><Check size={16} /> Done</button>}
        </footer>
      </section>
    </div>
  );
}

function ChevronArrow({ selected }: { selected: boolean }) {
  return selected ? <CheckCircle2 className="provider-picker-arrow" size={19} /> : <ArrowRight className="provider-picker-arrow" size={18} />;
}
