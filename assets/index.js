#!/bin/node
const OS = require("os");
const Process = require("process");
const ChildProcess = require("child_process");
const Path = require("path");

const arch = {
	"x64": "x86_64",
	"x32": "i686"
}[OS.arch()];
const platform = {
	"win32": "windows",
	"linux": "linux"
}[OS.platform()];
const ext = platform == "windows" ? ".exe" : "";

if (!arch || !platform) {
	console.error("Invalid platform.");
	Process.exit();
}

// Effectively become the target binary.
const process = ChildProcess.spawn(
	Path.resolve(__filename, `../bin/${arch}-${platform}${ext}`),
	{
		"stdio": "inherit"
	}
);
process.on("exit", code => Process.exit(code));
