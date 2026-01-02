import fs from 'fs';
import path from 'path';

const cargoPath = path.join(process.cwd(), 'jwt-tester-app', 'Cargo.toml');
const cargo = fs.readFileSync(cargoPath, 'utf8');
const match = cargo.match(/^version\s*=\s*"([^"]+)"/m);

if (!match) {
  console.error('Unable to find package version in Cargo.toml');
  process.exit(1);
}

process.stdout.write(match[1]);
