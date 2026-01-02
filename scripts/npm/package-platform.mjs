import fs from 'fs';
import path from 'path';

const args = {};
for (let i = 2; i < process.argv.length; i += 1) {
  const arg = process.argv[i];
  if (!arg.startsWith('--')) {
    continue;
  }
  const key = arg.slice(2);
  const value = process.argv[i + 1];
  if (!value || value.startsWith('--')) {
    console.error(`Missing value for --${key}`);
    process.exit(1);
  }
  args[key] = value;
  i += 1;
}

const required = ['variant', 'platform', 'arch', 'binary'];
for (const key of required) {
  if (!args[key]) {
    console.error(`Missing --${key}`);
    process.exit(1);
  }
}

const variant = args.variant;
const platform = args.platform;
const arch = args.arch;
let version = args.version;
const binaryPath = path.resolve(args.binary);
const assetsDir = args.assets ? path.resolve(args.assets) : null;

if (!version) {
  const cargoPath = path.join(process.cwd(), 'jwt-tester-app', 'Cargo.toml');
  const cargo = fs.readFileSync(cargoPath, 'utf8');
  const match = cargo.match(/^\s*version\s*=\s*\"([^\"]+)\"/m);
  if (!match) {
    console.error('Unable to find package version in Cargo.toml');
    process.exit(1);
  }
  version = match[1];
}

if (!fs.existsSync(binaryPath)) {
  console.error(`Binary not found at ${binaryPath}`);
  process.exit(1);
}

const scope = '@jakmer';
const baseName = variant === 'cli' ? 'jwt-tester-cli' : 'jwt-tester';
const packageName = `${scope}/${baseName}-${platform}-${arch}`;

const outDir = args.outDir
  ? path.resolve(args.outDir)
  : path.join(process.cwd(), 'dist', 'npm', scope, `${baseName}-${platform}-${arch}`);

fs.rmSync(outDir, { recursive: true, force: true });
fs.mkdirSync(path.join(outDir, 'bin'), { recursive: true });

const binName = platform === 'win32' ? 'jwt-tester.exe' : 'jwt-tester';
const binDest = path.join(outDir, 'bin', binName);
fs.copyFileSync(binaryPath, binDest);

const files = ['bin/**', 'README.md', 'LICENSE'];
if (variant === 'ui' && assetsDir) {
  const uiDest = path.join(outDir, 'ui', 'dist');
  fs.mkdirSync(uiDest, { recursive: true });
  fs.cpSync(assetsDir, uiDest, { recursive: true });
  files.push('ui/dist/**');
}

const pkg = {
  name: packageName,
  version,
  description: `Platform binary for ${baseName} (${platform} ${arch}).`,
  license: 'MIT',
  repository: {
    type: 'git',
    url: 'https://github.com/jMerta/jwt-tester.git',
  },
  homepage: 'https://github.com/jMerta/jwt-tester',
  bugs: {
    url: 'https://github.com/jMerta/jwt-tester/issues',
  },
  os: [platform],
  cpu: [arch],
  files,
  publishConfig: {
    access: 'public',
  },
};

fs.writeFileSync(path.join(outDir, 'package.json'), `${JSON.stringify(pkg, null, 2)}\n`);

const readme = `# ${packageName}\n\nPlatform binary package for ${baseName}.\n\nThis package is installed automatically via the \`${baseName}\` meta-package.\nDo not install it directly.\n`;
fs.writeFileSync(path.join(outDir, 'README.md'), readme);

const licenseSrc = path.join(process.cwd(), 'LICENSE');
if (fs.existsSync(licenseSrc)) {
  fs.copyFileSync(licenseSrc, path.join(outDir, 'LICENSE'));
}

console.log(outDir);
