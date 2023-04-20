-- Minimal wezterm configuration for our screenshots
local wezterm = require("wezterm")

return {
	default_prog = {
		"bash",
		"--norc",
		"-c",
		"mdcat --columns 50 ./sample/showcase.md",
	},
	term = "wezterm",
	font = wezterm.font("JetBrains Mono"),
	initial_cols = 60,
	-- We need 40 rows for the wrapped showcase document
	initial_rows = 40,
	exit_behavior = "Hold",
}
