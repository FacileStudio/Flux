vim.g.mapleader = " "
vim.g.maplocalleader = "\\"

local opt = vim.opt
opt.number = true
opt.signcolumn = "yes"
opt.termguicolors = true
opt.tabstop = 4
opt.shiftwidth = 4
opt.softtabstop = 4
opt.expandtab = true
opt.smartindent = true
opt.autoindent = true
opt.winborder = "rounded"
opt.ignorecase = true
opt.smartcase = true

vim.schedule(function()
	if vim.fn.has("clipboard") == 1 then
		vim.opt.clipboard = "unnamedplus"
	end
end)

vim.diagnostic.config({ underline = true, update_in_insert = false, severity_sort = true })

local map = vim.keymap.set
map("n", "<leader>w", "<cmd>write<CR>", { desc = "Write" })
map("n", "<leader>qq", "<cmd>quit<CR>", { desc = "Quit" })
map({ "n", "v", "x" }, "<leader>y", '"+y', { desc = "Yank to clipboard" })
map({ "n", "v", "x" }, "<leader>d", '"+d', { desc = "Delete to clipboard" })
map("n", "<leader>f", "<cmd>Telescope find_files<CR>", { desc = "Find files" })
map("n", "<leader><leader>", "<cmd>Telescope find_files<CR>", { desc = "Find files" })
map("n", "<leader>/", "<cmd>Telescope live_grep<CR>", { desc = "Grep" })
map("n", "<leader>h", "<cmd>Telescope help_tags<CR>", { desc = "Help" })
map("n", "<leader>b", "<cmd>Telescope buffers<CR>", { desc = "Buffers" })
map("n", "<leader>r", "<cmd>Telescope oldfiles<CR>", { desc = "Recent files" })
map("n", "<leader>x", "<cmd>bdelete<CR>", { desc = "Close buffer" })
map("n", "]d", function()
	vim.diagnostic.jump({ count = 1, float = true })
end, { desc = "Next diagnostic" })
map("n", "[d", function()
	vim.diagnostic.jump({ count = -1, float = true })
end, { desc = "Previous diagnostic" })
map("n", "<leader>cl", vim.diagnostic.open_float, { desc = "Line diagnostics" })
map("n", "<leader>cq", vim.diagnostic.setloclist, { desc = "Diagnostics list" })

vim.api.nvim_create_autocmd("LspAttach", {
	group = vim.api.nvim_create_augroup("flux-lsp", { clear = true }),
	callback = function(args)
		local function buf_map(lhs, rhs, desc)
			map("n", lhs, rhs, { buffer = args.buf, desc = desc })
		end
		buf_map("gd", vim.lsp.buf.definition, "Go to definition")
		buf_map("K", vim.lsp.buf.hover, "Hover")
		buf_map("<leader>rn", vim.lsp.buf.rename, "Rename")
	end,
})

local servers = {
	bashls = "bash-language-server",
	clangd = "clangd",
	gopls = "gopls",
	lua_ls = "lua-language-server",
	nil_ls = "nil",
	nixd = "nixd",
	pyright = "pyright-langserver",
	rust_analyzer = "rust-analyzer",
	svelte = "svelteserver",
	ts_ls = "typescript-language-server",
	zls = "zls",
}
for name, bin in pairs(servers) do
	if vim.fn.executable(bin) == 1 then
		vim.lsp.enable(name)
	end
end

vim.api.nvim_create_autocmd("FileType", {
	group = vim.api.nvim_create_augroup("flux-treesitter", { clear = true }),
	callback = function(args)
		local name = vim.api.nvim_buf_get_name(args.buf)
		local stat = name ~= "" and vim.uv.fs_stat(name)
		if stat and stat.size > 100 * 1024 then
			return
		end
		local lang = vim.treesitter.language.get_lang(args.match)
		if lang then
			pcall(vim.treesitter.start, args.buf, lang)
		end
	end,
})

local telescope = require("telescope")
local actions = require("telescope.actions")
telescope.setup({
	defaults = {
		mappings = {
			i = {
				["<C-j>"] = actions.move_selection_next,
				["<C-k>"] = actions.move_selection_previous,
			},
		},
	},
})

require("gitsigns").setup({
	signs = {
		add = { text = "│" },
		change = { text = "│" },
		delete = { text = "_" },
		topdelete = { text = "‾" },
		changedelete = { text = "~" },
	},
})

local keys = {}
for _, k in ipairs({ "Up", "Down", "Left", "Right", "CR", "Esc", "BS", "NL", "Space", "Tab", "ScrollWheelDown", "ScrollWheelUp" }) do
	keys[k] = "<" .. k .. "> "
end
for _, k in ipairs({ "C", "M", "D", "S" }) do
	keys[k] = "<" .. k .. "-…> "
end
for i = 1, 12 do
	keys["F" .. i] = "<F" .. i .. "> "
end
require("which-key").setup({ icons = { mappings = false, keys = keys } })

require("mini.surround").setup()
local notify = require("mini.notify")
notify.setup()
vim.notify = notify.make_notify()
