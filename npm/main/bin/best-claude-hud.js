#!/usr/bin/env node
const { spawnSync } = require('child_process');
const path = require('path');
const fs = require('fs');

const platform = process.platform;
const arch = process.arch;

let platformKey = `${platform}-${arch}`;
if (platform === 'linux' && arch === 'x64') {
  platformKey = 'linux-x64-musl';
}

const packageMap = {
  'darwin-x64': 'best-claude-hud-darwin-x64',
  'darwin-arm64': 'best-claude-hud-darwin-arm64',
  'linux-x64-musl': 'best-claude-hud-linux-x64-musl',
  'win32-x64': 'best-claude-hud-win32-x64',
  'win32-ia32': 'best-claude-hud-win32-x64', // Use 64-bit for 32-bit systems
};

const packageName = packageMap[platformKey];
if (!packageName) {
  console.error(`Error: Unsupported platform ${platformKey}`);
  console.error('Supported platforms: darwin (x64/arm64), linux (x64), win32 (x64)');
  console.error('Please visit https://github.com/GaoSSR/best-claude-hud for manual installation');
  process.exit(1);
}

const binaryName = platform === 'win32' ? 'best-claude-hud.exe' : 'best-claude-hud';
const possibleBinaryPaths = [
  path.join(__dirname, '..', 'node_modules', packageName, binaryName),
  (() => {
    try {
      const packageJson = require.resolve(`${packageName}/package.json`);
      return path.join(path.dirname(packageJson), binaryName);
    } catch {
      return null;
    }
  })(),
].filter(Boolean);

const binaryPath = possibleBinaryPaths.find((candidate) => fs.existsSync(candidate));

if (!binaryPath) {
  console.error(`Error: Binary package ${packageName} was not found.`);
  console.error('This might indicate a failed installation or unsupported platform.');
  console.error('Please try reinstalling: npm install -g best-claude-hud');
  process.exit(1);
}

const result = spawnSync(binaryPath, process.argv.slice(2), {
  stdio: 'inherit',
  shell: false
});

process.exit(result.status || 0);
