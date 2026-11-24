-- LSP profiles for this workspace (lsp-profiles.nvim). Switch with :LspProfile.
--
-- Each profile maps language server names (as in vim.lsp.config()) to config
-- that is merged over that server's own: settings, init_options, cmd, cmd_env, ...
-- Tables are merged key by key; anything else, including lists, replaces.
-- This file is data: it runs without access to `vim` or the standard library.
return {
    -- Check the web (wasm) build, like wasm.code-workspace in VS Code
    web = {
        rust_analyzer = {
            settings = {
                ['rust-analyzer'] = {
                    cargo = { target = 'wasm32-unknown-unknown' },
                    check = { targets = 'wasm32-unknown-unknown' },
                },
            },
        },
    },
}
