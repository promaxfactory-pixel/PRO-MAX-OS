import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const srcDir = path.join(root, "src");
const localesDir = path.join(srcDir, "i18n", "locales");

function walk(dir) {
  const out = [];
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    const full = path.join(dir, entry.name);
    if (entry.isDirectory()) {
      if (entry.name === "node_modules" || entry.name === "locales") continue;
      out.push(...walk(full));
    } else if (/\.(tsx|ts)$/.test(entry.name)) {
      out.push(full);
    }
  }
  return out;
}

function flatten(obj, prefix = "") {
  const out = [];
  for (const [key, value] of Object.entries(obj)) {
    const full = prefix ? `${prefix}.${key}` : key;
    if (value && typeof value === "object" && !Array.isArray(value)) {
      out.push(...flatten(value, full));
    } else {
      out.push(full);
    }
  }
  return out;
}

const usedKeys = new Set();
for (const file of walk(srcDir)) {
  const content = fs.readFileSync(file, "utf8");
  const re = /(?:^|[^.\w])t\(\s*["'`]([^"'`]+)["'`]\s*(?:[,)])/g;
  let m;
  while ((m = re.exec(content)) !== null) {
    const key = m[1].trim();
    if (key.includes(".") && !key.includes("${")) usedKeys.add(key);
  }
  const reI18n = /i18n\.t\(\s*["'`]([^"'`]+)["'`]\s*(?:[,)])/g;
  while ((m = reI18n.exec(content)) !== null) {
    const key = m[1].trim();
    if (key.includes(".") && !key.includes("${")) usedKeys.add(key);
  }
}

const en = JSON.parse(fs.readFileSync(path.join(localesDir, "en.json"), "utf8"));
const ar = JSON.parse(fs.readFileSync(path.join(localesDir, "ar.json"), "utf8"));
const enKeys = new Set(flatten(en));
const arKeys = new Set(flatten(ar));

let errors = 0;

for (const key of [...usedKeys].sort()) {
  if (!enKeys.has(key)) {
    console.error(`MISSING en: ${key}`);
    errors++;
  }
  if (!arKeys.has(key)) {
    console.error(`MISSING ar: ${key}`);
    errors++;
  }
}

for (const key of [...arKeys].sort()) {
  if (!enKeys.has(key)) {
    console.error(`ar-only key (missing in en): ${key}`);
    errors++;
  }
}

for (const key of [...enKeys].sort()) {
  if (!arKeys.has(key)) {
    console.error(`en-only key (missing in ar): ${key}`);
    errors++;
  }
}

console.log(`\nusedKeys in source: ${usedKeys.size}`);
console.log(`en keys: ${enKeys.size}, ar keys: ${arKeys.size}`);
console.log(errors === 0 ? "OK: i18n keys consistent" : `\n${errors} issue(s) found`);
process.exit(errors === 0 ? 0 : 1);
