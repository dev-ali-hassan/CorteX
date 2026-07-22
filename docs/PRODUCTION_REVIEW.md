# CorteX production review

## Decision

Keep the React + Tauri + SQLite architecture. It is a good fit for a fast Windows writing utility, and replacing it would add migration risk without improving the product. The production work should instead create clear boundaries around rewriting, persistence, desktop integration, and presentation.

## Findings that require correction

- Startup drops `rewrite_history`, permanently destroying user data.
- Rewrite requests have no input limit, timeout, cancellation identity, or output validation.
- Provider errors are flattened into strings and provider error bodies are assumed to be JSON.
- The popup stores errors in the output field, so an error can be copied or pasted into another app.
- Overlapping requests can publish stale output after the user changes mode or text.
- The backend and frontend both implement automatic copying.
- Offline normalization collapses line breaks and indentation.
- Main and popup surfaces expose different subsets of modes; `Expand` is absent.
- Selecting a mode can initiate work implicitly, which makes the interface feel unpredictable.
- Settings persistence is whole-document and can race during rapid changes.
- API credentials are persisted as plain JSON in SQLite. This is local, but not equivalent to secure credential storage.
- Accessibility is incomplete: dialogs do not trap focus and motion has no reduced-motion fallback.
- The main component and stylesheet are too concentrated, making regressions likely.
- The repository mixes npm and pnpm metadata and contains obsolete release-patching tooling and large local tool archives.

## Target architecture

1. `rewrite` service: validates requests, invokes one provider or offline fallback, validates structural invariants, records history, and returns a typed result.
2. `providers`: provider adapters with shared timeouts, safe error classification, and a single prompt contract.
3. `text`: line-preserving offline transformations. It must never claim translation when no translation engine exists.
4. `db`: additive schema migrations, settings persistence, and bounded rewrite history.
5. React request controller: latest-request-wins behavior, separate error/output state, explicit Rewrite/Copy/Replace actions, and consistent modes.
6. Desktop bridge: typed commands only; browser preview behavior stays isolated and cannot become a second product implementation.

## Compatibility policy

Existing settings deserialize with defaults. Existing provider IDs, shortcuts, theme choices, and the preview-first popup workflow remain valid. New fields and modes are additive. No source application text is replaced until the user explicitly chooses Replace in the panel.

## Known product truth

An offline rules engine cannot guarantee publication-quality correction for arbitrary language. CorteX will provide a useful local cleanup and label it honestly; configured AI providers receive the stronger editorial contract. Validation can guarantee structural safety (non-empty result, output limits, no conversational wrapper, and preservation checks), but it cannot mathematically prove that prose contains no linguistic errors.
