# Repository Guidelines

## Project Structure & Module Organization
Leafmate pairs a Vite-driven React UI with a Tauri shell. UI components live in `src/`; `main.tsx` wires the root `App.tsx`, while `MinimalViewer.tsx` hosts the EPUB workflow. Shared styling sits in `App.css`, `style.css`, and `src/assets/`. Static payloads (SVG icons, Vivliostyle resources, any bundled books) belong in `public/`. Native integrations reside in `src-tauri/`: Rust sources under `src-tauri/src`, configuration files (`tauri.conf.json`, `build.rs`), and packaged assets under `icons/` and `capabilities/`. Keep generated artifacts (`node_modules`, `src-tauri/target`) out of commits.

## Build, Test, and Development Commands
- `npm run dev`: launches the Vite dev server for the React UI; hot reloads TypeScript and CSS changes.
- `npm run tauri dev`: spawns the Tauri desktop shell and watches both Rust and Vite layers.
- `npm run build`: runs `tsc` for type-checking and produces a production bundle via `vite build`.
- `npm run preview`: serves the production build locally to sanity-check optimized assets.
- `cargo check --manifest-path src-tauri/Cargo.toml`: fast Rust compilation guard before opening a PR.

## Coding Style & Naming Conventions
TypeScript modules use ES imports, two-space indentation, and React functional components. Name components with `PascalCase` (`MinimalViewer`), hooks/utilities with `camelCase`, and co-locate feature helpers beside their component. Lean on explicit types from Readium/Vivliostyle packages before introducing `any`. Keep logging actionable; downgrade noisy traces to `console.debug` or remove before merge. Rust code targets edition 2021—run `cargo fmt` and `cargo clippy -- -D warnings` to maintain consistency.

## Testing Guidelines
Automated tests are not configured yet. When introducing logic, add Vitest suites under `src/__tests__/` (e.g., `MinimalViewer.test.tsx`) with Testing Library for DOM assertions, and mock Readium primitives. For native code, add integration checks in `src-tauri/tests` and run `cargo test`. Until harnesses exist, document manual smoke steps in the PR: open the default `moby-dick` manifest, navigate between chapters, and verify external link prompts on macOS and Windows builds.

## Commit & Pull Request Guidelines
Recent commits favor concise, imperative subjects (`Add feature to switch between chapters`). Follow that pattern, keep subjects under ~72 characters, and expand details in the body if cross-cutting changes occur. Reference issues with `Refs #123` or `Fixes #123` when applicable. Pull requests should explain the problem, outline the solution, list impacted UI states, and include screenshots or recordings for visible changes. Confirm `npm run build`, `npm run tauri dev`, and `cargo check` before requesting review, and call out follow-up tasks explicitly.

## Agent-Specific Instructions
When interacting with contributors or users through automation, respond in Japanese to match the repository’s communication tone. Internal comments can remain in English, but user-facing copy, review notes, and scripted prompts should default to Japanese.
