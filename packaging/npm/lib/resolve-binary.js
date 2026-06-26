"use strict";

const fs = require("node:fs");
const path = require("node:path");

const PLATFORMS = {
  "darwin-arm64": {
    alias: "best-claude-hud-darwin-arm64",
    binary: "best-claude-hud",
  },
  "darwin-x64": {
    alias: "best-claude-hud-darwin-x64",
    binary: "best-claude-hud",
  },
  "linux-x64": {
    alias: "best-claude-hud-linux-x64",
    binary: "best-claude-hud",
  },
  "win32-x64": {
    alias: "best-claude-hud-win32-x64",
    binary: "best-claude-hud.exe",
  },
};

function platformKey(platform = process.platform, arch = process.arch) {
  return `${platform}-${arch}`;
}

function resolvePackageJson(packageName) {
  try {
    return require.resolve(`${packageName}/package.json`);
  } catch {
    return null;
  }
}

function resolveBinary(options = {}) {
  const key = options.platformKey || platformKey(options.platform, options.arch);
  const platform = PLATFORMS[key];
  if (!platform) {
    throw new Error(
      `Unsupported platform ${key}. Supported platforms: darwin-arm64, darwin-x64, linux-x64, win32-x64.`
    );
  }

  const packageJsonPath = resolvePackageJson(platform.alias);
  if (!packageJsonPath) {
    throw new Error(
      `Missing optional dependency ${platform.alias}. Reinstall with: npm install -g best-claude-hud`
    );
  }

  const binaryPath = path.join(
    path.dirname(packageJsonPath),
    "vendor",
    key,
    platform.binary
  );
  if (!fs.existsSync(binaryPath)) {
    throw new Error(`Native binary not found: ${binaryPath}`);
  }

  return binaryPath;
}

module.exports = {
  PLATFORMS,
  platformKey,
  resolveBinary,
};
