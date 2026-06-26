#!/usr/bin/env node
"use strict";

const fs = require("node:fs");
const os = require("node:os");
const path = require("node:path");
const { execFileSync } = require("node:child_process");

const PACKAGE_ROOT = path.resolve(__dirname, "..");
const REPO_ROOT = path.resolve(PACKAGE_ROOT, "..", "..");
const PACKAGE_JSON = path.join(PACKAGE_ROOT, "package.json");

const PLATFORMS = [
  {
    key: "darwin-arm64",
    target: "aarch64-apple-darwin",
    alias: "best-claude-hud-darwin-arm64",
    os: "darwin",
    cpu: "arm64",
    archiveName: "best-claude-hud-darwin-arm64.tar.gz",
    binaryName: "best-claude-hud",
  },
  {
    key: "darwin-x64",
    target: "x86_64-apple-darwin",
    alias: "best-claude-hud-darwin-x64",
    os: "darwin",
    cpu: "x64",
    archiveName: "best-claude-hud-darwin-x64.tar.gz",
    binaryName: "best-claude-hud",
  },
  {
    key: "linux-x64",
    target: "x86_64-unknown-linux-musl",
    alias: "best-claude-hud-linux-x64",
    os: "linux",
    cpu: "x64",
    archiveName: "best-claude-hud-linux-x64-musl.tar.gz",
    binaryName: "best-claude-hud",
  },
  {
    key: "win32-x64",
    target: "x86_64-pc-windows-msvc",
    alias: "best-claude-hud-win32-x64",
    os: "win32",
    cpu: "x64",
    archiveName: "best-claude-hud-win32-x64.zip",
    binaryName: "best-claude-hud.exe",
  },
];

function parseArgs(argv) {
  const args = {};
  for (let index = 2; index < argv.length; index += 1) {
    const name = argv[index];
    const value = argv[index + 1];
    if (!name.startsWith("--") || value === undefined || value.startsWith("--")) {
      throw new Error(`missing value for ${name}`);
    }
    args[name.slice(2)] = value;
    index += 1;
  }

  for (const required of ["version", "release-dir", "output-dir"]) {
    if (!args[required]) {
      throw new Error(`missing --${required}`);
    }
  }

  return {
    version: args.version,
    releaseDir: path.resolve(args["release-dir"]),
    outputDir: path.resolve(args["output-dir"]),
  };
}

function readPackageJson() {
  return JSON.parse(fs.readFileSync(PACKAGE_JSON, "utf8"));
}

function copyIfExists(source, destination) {
  if (fs.existsSync(source)) {
    fs.copyFileSync(source, destination);
  }
}

function writeJson(filePath, value) {
  fs.writeFileSync(filePath, `${JSON.stringify(value, null, 2)}\n`);
}

function prepareEmptyDir(prefix) {
  return fs.mkdtempSync(path.join(os.tmpdir(), prefix));
}

function copyPackageDocs(stagingDir) {
  copyIfExists(path.join(REPO_ROOT, "LICENSE"), path.join(stagingDir, "LICENSE"));
  copyIfExists(path.join(PACKAGE_ROOT, "README.md"), path.join(stagingDir, "README.md"));
}

function stageRootPackage(version) {
  const stagingDir = prepareEmptyDir("best-claude-hud-npm-root-");
  const sourcePackage = readPackageJson();
  const packageJson = {
    ...sourcePackage,
    version,
    private: undefined,
    scripts: undefined,
    files: ["LICENSE", "README.md", "bin/", "lib/"],
    optionalDependencies: Object.fromEntries(
      PLATFORMS.map((platform) => [
        platform.alias,
        `npm:${sourcePackage.name}@${version}-${platform.key}`,
      ])
    ),
  };
  delete packageJson.private;
  delete packageJson.scripts;

  fs.mkdirSync(path.join(stagingDir, "bin"), { recursive: true });
  fs.mkdirSync(path.join(stagingDir, "lib"), { recursive: true });
  fs.copyFileSync(
    path.join(PACKAGE_ROOT, "bin", "best-claude-hud.js"),
    path.join(stagingDir, "bin", "best-claude-hud.js")
  );
  fs.copyFileSync(
    path.join(PACKAGE_ROOT, "lib", "resolve-binary.js"),
    path.join(stagingDir, "lib", "resolve-binary.js")
  );
  copyPackageDocs(stagingDir);
  writeJson(path.join(stagingDir, "package.json"), packageJson);
  return stagingDir;
}

function stagePlatformPackage(version, platform, releaseDir) {
  const archivePath = path.join(releaseDir, platform.archiveName);
  if (!fs.existsSync(archivePath)) {
    throw new Error(`missing release archive: ${archivePath}`);
  }

  const stagingDir = prepareEmptyDir(`best-claude-hud-npm-${platform.key}-`);
  const extractedDir = prepareEmptyDir(`best-claude-hud-release-${platform.key}-`);
  if (platform.archiveName.endsWith(".zip")) {
    execFileSync("unzip", ["-q", archivePath, "-d", extractedDir]);
  } else {
    execFileSync("tar", ["-xzf", archivePath, "-C", extractedDir]);
  }

  const sourceBinary = path.join(extractedDir, platform.binaryName);
  if (!fs.existsSync(sourceBinary)) {
    throw new Error(
      `release archive did not contain ${platform.binaryName}: ${archivePath}`
    );
  }

  const vendorDir = path.join(stagingDir, "vendor", platform.key);
  fs.mkdirSync(vendorDir, { recursive: true });
  const binaryDest = path.join(vendorDir, platform.binaryName);
  fs.copyFileSync(sourceBinary, binaryDest);
  fs.chmodSync(binaryDest, 0o755);

  const sourcePackage = readPackageJson();
  const packageJson = {
    name: sourcePackage.name,
    version: `${version}-${platform.key}`,
    description: `${sourcePackage.description} (${platform.key})`,
    license: sourcePackage.license,
    os: [platform.os],
    cpu: [platform.cpu],
    files: ["LICENSE", "README.md", "vendor/"],
    engines: sourcePackage.engines,
    repository: sourcePackage.repository,
  };

  copyPackageDocs(stagingDir);
  writeJson(path.join(stagingDir, "package.json"), packageJson);
  return stagingDir;
}

function npmPack(stagingDir, outputDir, outputName) {
  fs.mkdirSync(outputDir, { recursive: true });
  const packDir = prepareEmptyDir("best-claude-hud-npm-pack-");
  const output = execFileSync(
    "npm",
    ["pack", "--json", "--pack-destination", packDir],
    {
      cwd: stagingDir,
      encoding: "utf8",
      env: {
        ...process.env,
        NPM_CONFIG_CACHE: path.join(packDir, "cache"),
        NPM_CONFIG_LOGS_DIR: path.join(packDir, "logs"),
      },
    }
  );
  const packed = JSON.parse(output);
  if (!Array.isArray(packed) || !packed[0] || !packed[0].filename) {
    throw new Error("npm pack did not report a tarball filename");
  }

  const generatedPath = path.join(packDir, packed[0].filename);
  const finalPath = path.join(outputDir, outputName);
  fs.renameSync(generatedPath, finalPath);
  return finalPath;
}

function main() {
  const args = parseArgs(process.argv);
  const outputs = [];

  const rootStaging = stageRootPackage(args.version);
  outputs.push(
    npmPack(rootStaging, args.outputDir, `best-claude-hud-npm-${args.version}.tgz`)
  );

  for (const platform of PLATFORMS) {
    const staging = stagePlatformPackage(args.version, platform, args.releaseDir);
    outputs.push(
      npmPack(
        staging,
        args.outputDir,
        `best-claude-hud-npm-${platform.key}-${args.version}.tgz`
      )
    );
  }

  for (const output of outputs) {
    console.log(output);
  }
}

main();
