require("vis")

vis.events.subscribe(vis.events.INIT, function()
	vis:command("set autoindent on")
	vis:command("set ignorecase on")
end)

vis.events.subscribe(vis.events.WIN_OPEN, function(win)
	win.options.numbers = true
	win.options.tabwidth = 4
	win.options.expandtab = true
end)

local MODE_STYLE = 60

local mode_names = {
	[vis.modes.NORMAL] = " NORMAL ",
	[vis.modes.INSERT] = " INSERT ",
	[vis.modes.VISUAL] = " VISUAL ",
	[vis.modes.VISUAL_LINE] = " V-LINE ",
	[vis.modes.REPLACE] = " REPLACE ",
	[vis.modes.OPERATOR_PENDING] = " PENDING ",
}

local mode_colors = {
	[vis.modes.NORMAL] = "blue",
	[vis.modes.INSERT] = "green",
	[vis.modes.VISUAL] = "magenta",
	[vis.modes.VISUAL_LINE] = "magenta",
	[vis.modes.REPLACE] = "red",
	[vis.modes.OPERATOR_PENDING] = "yellow",
}

vis.events.subscribe(vis.events.WIN_STATUS, function(win)
	local name = win.file.name or "[No Name]"
	local mod = win.file.modified and " [+]" or ""
	local right = string.format("%s  │  %d:%d", win.syntax or "text", win.selection.line, win.selection.col)

	if win ~= vis.win then
		win:status(" " .. name .. mod, right)
		return true
	end

	local mode = mode_names[vis.mode] or " NORMAL "
	win:status(mode .. " " .. name .. mod, right)
	local style = "fore:" .. (mode_colors[vis.mode] or "blue") .. ",reverse,bold"
	if vis.ui and vis.ui.style_define then
		vis.ui:style_define(MODE_STYLE, style)
	elseif win.style_define then
		win:style_define(MODE_STYLE, style)
	end
	for x = 0, #mode - 1 do
		win:style_pos(MODE_STYLE, x, win.height - 1)
	end
	return true
end)

local function has_cmd(cmd)
	local res = os.execute("command -v " .. cmd .. " >/dev/null 2>&1")
	return res == true or res == 0
end

local function fzf_run(pipeline)
	local tmp = os.tmpname()
	vis:command(string.format("! %s > %q", pipeline, tmp))
	vis:feedkeys("<C-l>")
	local f = io.open(tmp, "r")
	if not f then
		return nil
	end
	local res = f:read("*l")
	f:close()
	os.remove(tmp)
	return res
end

local function fzf_files()
	local file_cmd = has_cmd("fd") and "fd --type f --hidden --exclude .git 2>/dev/null"
		or "find . -type f \\( -name .git -prune -o -print \\) 2>/dev/null"
	local selected = fzf_run(string.format("%s | fzf --prompt='Files> ' --height=40%% --reverse", file_cmd))
	if selected and selected ~= "" then
		vis:command(string.format("e %q", selected))
	end
end

local function fzf_grep()
	local cmd = "rg --column --line-number --no-heading --smart-case"
	local selected = fzf_run(
		string.format(
			"fzf --disabled --prompt='Grep> ' --height=40%% --reverse --bind 'start:reload:%s \"\"' --bind 'change:reload:%s {q} || true'",
			cmd,
			cmd
		)
	)
	if selected and selected ~= "" then
		local file, line = selected:match("^([^:]+):(%d+)")
		if file and line then
			vis:command(string.format("e %q", file))
			vis:command(line)
		end
	end
end

local function fzf_buffers()
	local tmp_in = os.tmpname()
	local fin = io.open(tmp_in, "w")
	if not fin then
		return
	end
	local names = {}
	local count = 0
	for win in vis:windows() do
		local n = win.file and win.file.name or "[No Name]"
		local m = (win.file and win.file.modified) and " [+]" or ""
		count = count + 1
		local entry = string.format("[%d] %s%s", count, n, m)
		fin:write(entry .. "\n")
		names[entry] = win.file and win.file.name
	end
	fin:close()
	local selected = fzf_run(string.format("fzf --prompt='Buffers> ' --height=30%% --reverse < %q", tmp_in))
	os.remove(tmp_in)
	if selected and names[selected] then
		vis:command(string.format("e %q", names[selected]))
	end
end

vis:map(vis.modes.NORMAL, "<Space>y", '"+y')
vis:map(vis.modes.VISUAL, "<Space>y", '"+y')
vis:map(vis.modes.NORMAL, "<Space>p", '"+p')
vis:map(vis.modes.NORMAL, "<Space>P", '"+P')
vis:map(vis.modes.NORMAL, "<Space>f", fzf_files)
vis:map(vis.modes.NORMAL, "<Space>/", fzf_grep)
vis:map(vis.modes.NORMAL, "<Space>b", fzf_buffers)

vis:command_register("fzf", fzf_files, "Fuzzy find files")
vis:command_register("grep", fzf_grep, "Live grep project")
vis:command_register("buffers", fzf_buffers, "Switch buffers")
