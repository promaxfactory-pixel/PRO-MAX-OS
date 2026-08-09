import { execSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";

const root = process.cwd();
const [,, ns, ...files] = process.argv;
const outPath = path.join(root, "scripts", ".i18n-batch", `${process.env.OUT || ns}.json`);

const arabic = /[\u0600-\u06FF]/;
const keyRe = /\bt\(\s*["'`]([^"'`]+)["'`]/;
const strRe = /"([^"]*[\u0600-\u06FF][^"]*)"/;

const pairs = new Map(); // key -> { ar, file }

function handleHunk(hunkLines, file) {
  const removedArabic = [];
  const addedKeys = [];
  for (const line of hunkLines) {
    if (line.startsWith("+") && !line.startsWith("+++")) {
      const m = line.match(keyRe);
      if (m) addedKeys.push(m[1]);
    } else if (line.startsWith("-") && !line.startsWith("---") && arabic.test(line)) {
      const m = line.match(strRe);
      if (m) removedArabic.push(m[1]);
    }
  }
  let di = 0;
  for (const k of addedKeys) {
    if (!k.startsWith(ns)) continue;
    if (di < removedArabic.length) {
      if (!pairs.has(k)) pairs.set(k, { ar: removedArabic[di], file });
      di++;
    } else if (!pairs.has(k)) {
      pairs.set(k, { ar: null, file });
    }
  }
}

for (const file of files) {
  const diff = execSync(`git diff -- ${file}`, { cwd: root, encoding: "utf8", maxBuffer: 64 * 1024 * 1024 });
  const hunks = diff.split(/\n@@[^@]*@@\n/);
  for (const hunk of hunks) {
    const lines = hunk.split("\n");
    handleHunk(lines.filter((l) => l.startsWith("+") || l.startsWith("-")), file);
  }
}

const out = {};
for (const [key, { ar, file }] of pairs) {
  const keyPath = key.split(".");
  let cur = out;
  for (let i = 1; i < keyPath.length - 1; i++) {
    const k = keyPath[i];
    if (!cur[k]) cur[k] = {};
    cur = cur[k];
  }
  const leaf = keyPath[keyPath.length - 1];
  if (!cur[leaf]) cur[leaf] = { en: "", ar: ar || "", _file: file };
}

fs.writeFileSync(outPath, JSON.stringify(out, null, 2) + "\n");
const flat = [];
const walk = (obj, prefix) => {
  for (const [k, v] of Object.entries(obj)) {
    if (v && typeof v === "object" && "ar" in v) flat.push(`${prefix}${k}`);
    else if (v && typeof v === "object") walk(v, `${prefix}${k}.`);
  }
};
walk(out, `${ns}.`);
console.log(`Wrote ${outPath} with ${flat.length} keys:`);
for (const k of flat) console.log(k);
