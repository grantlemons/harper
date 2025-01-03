---
title: Neovim
---

[Neovim](https://neovim.io/) is a popular open-source text editor.
Its lack of fast grammar-checking was the primary motivation for creating Harper.

## Installation

How you choose to install `harper-ls` depends on your use-case.
Right now, we only directly support usage through [`nvim-lspconfig`](https://github.com/neovim/nvim-lspconfig/blob/master/doc/server_configurations.md#harper_ls).
Refer to the linked documentation for more information.

If you happen to use [`mason.nvim`](https://github.com/williamboman/mason.nvim), installation will be pretty straightforward.
`harper-ls` is in the official Mason registry, so you can install it the same way you install anything through Mason.

If you **don't** install your LSPs through Mason, we have binary releases available on [GitHub](https://github.com/Automattic/harper/releases) or you can use one of a number of package managers.

### Cargo

If you have [Rust installed](https://www.rust-lang.org/tools/install), you're in luck!
To install `harper-ls`, simply run:

```bash
cargo install harper-ls --locked
```

### Arch Linux

Harper is available through the `extra` repo:

```bash
sudo pacman -S harper
```

## Configuration

Neovim is also one of the two primarily supported editors for `harper-ls`.
As such, you can view this page as canonical documentation for the available configuration options.
[Helix](./helix) and [Zed](./zed) users may find it helpful.

### Dictionaries

You do not have to stick with the default dictionary locations ([listed on this page](./language-server)).
If you use Neovim, you can set the location of the user dictionary with the `userDictPath` key, and the file dictionary with the `fileDictPath` key:

```lua
lspconfig.harper_ls.setup {
  settings = {
    ["harper-ls"] = {
      userDictPath = "~/dict.txt",
      fileDictPath = "~/.harper/",
    }
  },
}
```

### Linters

Linters are grammatical rules Harper looks for to correct your work.
You can toggle them on or off to your liking.

```lua
lspconfig.harper_ls.setup {
  settings = {
    ["harper-ls"] = {
      linters = {
        spell_check = true,
        spelled_numbers = false,
        an_a = true,
        sentence_capitalization = true,
        unclosed_quotes = true,
        wrong_quotes = false,
        long_sentences = true,
        repeated_words = true,
        spaces = true,
        matcher = true,
        correct_number_suffix = true,
        number_suffix_capitalization = true,
        multiple_sequential_pronouns = true,
        linking_verbs = false,
        avoid_curses = true,
        terminating_conjunctions = true
      }
    }
  },
}
```

By default, `harper-ls` will mark all diagnostics with HINT.
If you want to configure this, refer below:

```lua
lspconfig.harper_ls.setup {
  settings = {
    ["harper-ls"] = {
        diagnosticSeverity = "hint" -- Can also be "information", "warning", or "error"
    }
  },
}
```

You can also configure how `harper-ls` displays code actions.
For example, to make code actions appear in "stable" positions, use the following configuration:

```lua
lspconfig.harper_ls.setup {
  settings = {
    ["harper-ls"] = {
      codeActions = {
        forceStable = true
      }
    }
  },
}
```
