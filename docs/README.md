# Kensaku documentation

Start with the [README](../README.md) for what Kensaku is and how to install it.
These pages go deeper.

## Using Kensaku

| Page | What it covers |
|---|---|
| [USAGE.md](USAGE.md) | Window modes, keyboard, query language, plain-language intent, settings, indexing behavior, slash commands. |
| [INSTALL.md](INSTALL.md) | Prebuilt bundles, unsigned-build warnings, build prerequisites, the agent CLI, OCR engines, first run, uninstall. |
| [CONFIG.md](CONFIG.md) | Every `config.json` key with its real default, data-directory overrides, migrations. |
| [LANGUAGES.md](LANGUAGES.md) | CJK tokenization, code/data formats, document formats, OCR language support. |
| [FAQ.md](FAQ.md) | Privacy, memory, OCR, updates, agents, and common "why no results?" questions. |
| [PRIVACY.md](PRIVACY.md) | What stays local, what leaves the machine, and what the MCP server can read. |

## For agents and integrations

| Page | What it covers |
|---|---|
| [AGENTS.md](AGENTS.md) | MCP setup, tool schemas, pagination, scoped roots, CLI, and the read-only contract. |
| [EXTRACTORS.md](EXTRACTORS.md) | Opt-in external extractors for formats Kensaku does not decode natively. |

## For contributors

| Page | What it covers |
|---|---|
| [ARCHITECTURE.md](ARCHITECTURE.md) | Process model, storage, index tiers, query pipeline, extraction, GUI, updates, module map. |
| [BENCHMARKS.md](BENCHMARKS.md) | Harnesses, methodology, 100k Windows/Linux reference numbers, indexing/extraction measurements, caveats. |
| [CONTRIBUTING.md](CONTRIBUTING.md) | Setup, the checks CI runs, test expectations, style, performance changes. |
| [RELEASING.md](RELEASING.md) | Version bump, CI build, draft release review, asset checklist, signing notes. |

## Conventions in these docs

- **Shipped** means the code in the current `1.0.16` tree does it.
- **Reference** numbers are measured on the stated machine and corpus. They are
  not guarantees; see [BENCHMARKS.md](BENCHMARKS.md) for the caveats.
- **Not shipped** is called out explicitly. Visual search, native OCR, and
  bundled LLM/chat are not in the production path.

Planning documents and internal experiment logs were removed for the 1.0.16
documentation pass; the shipped behavior lives in the pages above.
