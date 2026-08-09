import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const batchDir = path.join(root, "scripts", ".i18n-batch");
const enPath = path.join(root, "src", "i18n", "locales", "en.json");
const arPath = path.join(root, "src", "i18n", "locales", "ar.json");

function readJson(p) {
  const raw = fs.readFileSync(p, "utf8").replace(/^\uFEFF/, "");
  return JSON.parse(raw);
}

function setDeep(obj, keys, value) {
  let cur = obj;
  for (let i = 0; i < keys.length - 1; i++) {
    const k = keys[i];
    if (!cur[k] || typeof cur[k] !== "object") cur[k] = {};
    cur = cur[k];
  }
  cur[keys[keys.length - 1]] = value;
}

if (!fs.existsSync(batchDir)) {
  console.log("No batch dir, nothing to merge");
  process.exit(0);
}

const batchFiles = fs.readdirSync(batchDir).filter((f) => f.endsWith(".json")).sort();
if (batchFiles.length === 0) {
  console.log("No batch files found, nothing to merge");
  process.exit(0);
}

const en = readJson(enPath);
const ar = readJson(arPath);

const conflicts = [];
let added = 0;

function isLeaf(v) {
  return v && typeof v === "object" && !Array.isArray(v) && typeof v.en === "string" && typeof v.ar === "string";
}

function mergeNode(target, ns, key, value, file, lang) {
  if (!target[ns] || typeof target[ns] !== "object") target[ns] = {};
  if (isLeaf(value)) {
    const str = lang === "en" ? value.en : value.ar;
    const existing = target[ns]?.[key];
    if (existing !== undefined && existing !== str) {
      const rec = { file, ns, key, [lang === "en" ? "oldEn" : "oldAr"]: existing, [lang === "en" ? "newEn" : "newAr"]: str };
      conflicts.push(rec);
    }
    target[ns][key] = str;
    added++;
    return;
  }
  if (value && typeof value === "object" && !Array.isArray(value)) {
    for (const [childKey, childVal] of Object.entries(value)) {
      mergeNode(target[ns], key, childKey, childVal, file, lang);
    }
    return;
  }
  console.error(`Batch ${file}: key ${ns}.${key} has invalid shape (expected { en, ar } or nested object)`);
}

for (const file of batchFiles) {
  const data = readJson(path.join(batchDir, file));
  for (const [ns, entries] of Object.entries(data)) {
    if (!en[ns] || typeof en[ns] !== "object") en[ns] = {};
    if (!ar[ns] || typeof ar[ns] !== "object") ar[ns] = {};
    for (const [key, value] of Object.entries(entries)) {
      mergeNode(en, ns, key, value, file, "en");
      mergeNode(ar, ns, key, value, file, "ar");
    }
  }
}

fs.writeFileSync(enPath, JSON.stringify(en, null, 2) + "\n");
fs.writeFileSync(arPath, JSON.stringify(ar, null, 2) + "\n");

console.log(`Merged ${added} keys from ${batchFiles.length} batch file(s) into en.json + ar.json`);
if (conflicts.length > 0) {
  console.warn(`${conflicts.length} value conflict(s) — later batches won, review if needed:`);
  for (const c of conflicts.slice(0, 20)) console.warn(JSON.stringify(c));
}
