"use strict";

const assert = require("node:assert/strict");
const test = require("node:test");
const { PLATFORMS, platformKey } = require("../lib/resolve-binary");

test("platform keys match supported aliases", () => {
  assert.deepEqual(Object.keys(PLATFORMS).sort(), [
    "darwin-arm64",
    "darwin-x64",
    "linux-x64",
    "win32-x64",
  ]);
});

test("platformKey combines node platform and arch", () => {
  assert.equal(platformKey("darwin", "arm64"), "darwin-arm64");
  assert.equal(platformKey("linux", "x64"), "linux-x64");
});
