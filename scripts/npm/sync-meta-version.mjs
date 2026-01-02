import fs from 'fs';
import path from 'path';

const cargoPath = path.join(process.cwd(), 'jwt-tester-app', 'Cargo.toml');
const cargo = fs.readFileSync(cargoPath, 'utf8');
const match = cargo.match(/^version\s*=\s*"([^"]+)"/m);

if (!match) {
  console.error('Unable to find package version in Cargo.toml');
  process.exit(1);
}

const version = match[1];
const metaPackages = [
  path.join(process.cwd(), 'npm', 'jwt-tester', 'package.json'),
  path.join(process.cwd(), 'npm', 'jwt-tester-cli', 'package.json'),
];

for (const pkgPath of metaPackages) {
  const data = JSON.parse(fs.readFileSync(pkgPath, 'utf8'));
  data.version = version;
  if (data.optionalDependencies) {
    for (const key of Object.keys(data.optionalDependencies)) {
      data.optionalDependencies[key] = version;
    }
  }
  fs.writeFileSync(pkgPath, `${JSON.stringify(data, null, 2)}\n`);
}

console.log(version);
