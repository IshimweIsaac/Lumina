# Lumina VS Code Extension

Provides syntax highlighting, syntax configuration (bracket matching and auto-closing), and code snippets for Lumina source files (`.lum`).

## Features
- Full grammar parsing with TextMate (`.tmLanguage.json`)
- Multi-line support for interpolation logic and keywords
- Auto-completions for keywords via `.json` snippets

## Developing
This extension is declarative so it does not contain a language server.
If you open `.lum` files and see formatting and colored scopes, then it is behaving properly.
