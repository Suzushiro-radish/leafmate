# Repository Guidelines

## Project Structure & Module Organization
Leafmate pairs a Vite-driven React UI with a Tauri shell. UI components live in `src/`; `main.tsx` wires the root `App.tsx`, while `MinimalViewer.tsx` hosts the EPUB workflow. Shared styling sits in `App.css`, `style.css`, and `src/assets/`. Static payloads (SVG icons, Vivliostyle resources, any bundled books) belong in `public/`. Native integrations reside in `src-tauri/`: Rust sources under `src-tauri/src`, configuration files (`tauri.conf.json`, `build.rs`), and packaged assets under `icons/` and `capabilities/`. Keep generated artifacts (`node_modules`, `src-tauri/target`) out of commits.

## Build, Test, and Development Commands
**Package Manager**: This project uses `pnpm`. Do NOT use `npm` or `yarn`.

- `pnpm dev`: launches the Vite dev server for the React UI; hot reloads TypeScript and CSS changes.
- `pnpm tauri dev`: spawns the Tauri desktop shell and watches both Rust and Vite layers.
- `pnpm build`: runs `tsc` for type-checking and produces a production bundle via `vite build`.
- `pnpm preview`: serves the production build locally to sanity-check optimized assets.
- `pnpm test`: runs Vitest tests for TypeScript code.
- `cargo check --manifest-path src-tauri/Cargo.toml`: fast Rust compilation guard before opening a PR.
- `cargo test --manifest-path src-tauri/Cargo.toml`: runs Rust tests.

## Coding Style & Naming Conventions
TypeScript modules use ES imports, two-space indentation, and React functional components. Name components with `PascalCase` (`MinimalViewer`), hooks/utilities with `camelCase`, and co-locate feature helpers beside their component. Lean on explicit types from Readium/Vivliostyle packages before introducing `any`. Keep logging actionable; downgrade noisy traces to `console.debug` or remove before merge. Rust code targets edition 2021—run `cargo fmt` and `cargo clippy -- -D warnings` to maintain consistency.

## Testing Guidelines
TypeScript: Use Vitest for unit tests. Place tests alongside the code (e.g., `TauriFetcher.test.ts`). Run `pnpm test` to execute tests. Mock Tauri commands using `vi.mock("@tauri-apps/api/core")`. Rust: Add tests in `src-tauri/src/` modules or `src-tauri/tests/`. Run `cargo test --manifest-path src-tauri/Cargo.toml`.

## Commit & Pull Request Guidelines
Recent commits favor concise, imperative subjects (`Add feature to switch between chapters`). Follow that pattern, keep subjects under ~72 characters, and expand details in the body if cross-cutting changes occur. Reference issues with `Refs #123` or `Fixes #123` when applicable. Pull requests should explain the problem, outline the solution, list impacted UI states, and include screenshots or recordings for visible changes. Confirm `pnpm build`, `pnpm test`, `cargo check`, and `cargo test` before requesting review, and call out follow-up tasks explicitly.

## Agent-Specific Instructions
When interacting with contributors or users through automation, respond in Japanese to match the repository’s communication tone. Internal comments can remain in English, but user-facing copy, review notes, and scripted prompts should default to Japanese.
