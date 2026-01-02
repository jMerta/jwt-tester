import fs from 'fs';
import path from 'path';

const tag = process.env.GITHUB_REF_NAME || '';
if (!tag) {
  console.error('GITHUB_REF_NAME is not set.');
  process.exit(1);
}

const cargoPath = path.join(process.cwd(), 'jwt-tester-app', 'Cargo.toml');
const cargo = fs.readFileSync(cargoPath, 'utf8');
const match = cargo.match(/^version\s*=\s*"([^"]+)"/m);

if (!match) {
  console.error('Unable to find package version in Cargo.toml');
  process.exit(1);
}

const version = match[1];
const normalizedTag = tag.startsWith('v') ? tag.slice(1) : tag;

if (normalizedTag !== version) {
  console.error(`Release tag (${tag}) does not match Cargo.toml version (${version}).`);
  process.exit(1);
}

console.log(`Release tag ${tag} matches Cargo.toml version ${version}.`);
