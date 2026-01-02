#!/usr/bin/env node
'use strict';

const path = require('path');
const { spawnSync } = require('child_process');

const variant = 'ui';
const packageBase = '@jakmer/jwt-tester';
const supported = new Set([
  'darwin-x64',
  'darwin-arm64',
  'linux-x64',
  'linux-arm64',
  'win32-x64',
]);

const platform = process.platform;
const arch = process.arch;
const platformKey = `${platform}-${arch}`;

if (!supported.has(platformKey)) {
  console.error(`jwt-tester: unsupported platform ${platform}/${arch}.`);
  console.error('Supported platforms: darwin-x64, darwin-arm64, linux-x64, win32-x64.');
  process.exit(1);
}

const packageName = `${packageBase}-${platform}-${arch}`;
let packageRoot;
try {
  packageRoot = path.dirname(require.resolve(`${packageName}/package.json`));
} catch (err) {
  console.error(`jwt-tester: platform package not found (${packageName}).`);
  console.error('Try reinstalling or verify npm optionalDependencies installed correctly.');
  process.exit(1);
}

const binName = platform === 'win32' ? 'jwt-tester.exe' : 'jwt-tester';
const binPath = path.join(packageRoot, 'bin', binName);

const env = { ...process.env };
if (!env.JWT_TESTER_UI_ASSETS_DIR) {
  env.JWT_TESTER_UI_ASSETS_DIR = path.join(packageRoot, 'ui', 'dist');
}

const result = spawnSync(binPath, process.argv.slice(2), {
  stdio: 'inherit',
  env,
});

if (result.error) {
  console.error(`jwt-tester: failed to launch ${binName}: ${result.error.message}`);
  process.exit(1);
}

process.exit(result.status ?? 1);
