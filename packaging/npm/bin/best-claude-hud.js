#!/usr/bin/env node
"use strict";

const { spawnSync } = require("node:child_process");
const { resolveBinary } = require("../lib/resolve-binary");

const binaryPath = resolveBinary();
const result = spawnSync(binaryPath, process.argv.slice(2), {
  stdio: "inherit",
  shell: false,
});

if (result.error) {
  console.error(result.error.message);
  process.exit(1);
}

process.exit(result.status ?? 0);
