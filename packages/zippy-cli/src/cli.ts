#!/usr/bin/env bun

import { spawnSync } from "child_process";
import { join, relative, dirname, basename, extname } from "path";
import { readdirSync, readFileSync, writeFileSync, existsSync, mkdirSync, statSync, copyFileSync, unlinkSync, renameSync, rmdirSync } from "fs";
import { createHash } from "crypto";

const [command, ...args] = process.argv.slice(2);

switch (command) {
  case "dev":
    await import("./dev");
    break;
  case "build":
    await build(args[0] || "src");
    break;
  case "init":
    await init();
    break;
  default:
    console.log(`
Zippy CLI v0.0.1

Usage:
  zippy dev          Start dev server with hot reload
  zippy build [dir]  Build project for production
  zippy init         Create a new Zippy project
`);
}

async function build(dir: string) {
  const outDir = "dist";
  if (existsSync(outDir)) {
    for (const f of walk(outDir)) {
      try { unlinkSync(f); } catch {}
    }
  }
  if (!existsSync(outDir)) mkdirSync(outDir, { recursive: true });

  const entries = walk(dir);
  const zippyFiles = entries.filter(f => f.endsWith(".zippy"));

  if (zippyFiles.length === 0) {
    console.log(" No .zippy files found in", dir);
    return;
  }

  const compilerPath = findCompiler();
  if (!compilerPath) {
    console.error(" zippy-compiler not found. Build it first: cd packages/compiler && cargo build");
    process.exit(1);
  }

  for (const file of zippyFiles) {
    const rel = relative(dir, file);
    const outFile = join(outDir, rel.replace(/\.zippy$/, ".js"));
    const outParent = dirname(outFile);
    if (!existsSync(outParent)) mkdirSync(outParent, { recursive: true });

    console.log(` ${file} -> ${outFile}`);
    const result = spawnSync(compilerPath, [file, outFile], { stdio: "inherit" });
    if (result.status !== 0) {
      console.error(` Failed to compile ${file}`);
      process.exit(1);
    }
  }

  console.log("\n Extracting CSS...");
  const cssBlocks: string[] = [];
  const cssRegex = /const __style = document\.createElement\('style'\);\s*__style\.textContent = `([^`]+)`;\s*document\.head\.append\(__style\);/g;
  for (const jsFile of walk(outDir).filter(f => f.endsWith(".js") && !f.endsWith(".min.js"))) {
    const content = readFileSync(jsFile, "utf-8");
    const matches = [...content.matchAll(cssRegex)];
    if (matches.length > 0) {
      for (const m of matches) cssBlocks.push(m[1]);
      const cleaned = content.replace(cssRegex, "/* CSS moved to styles.css */");
      writeFileSync(jsFile, cleaned);
    }
  }
  if (cssBlocks.length > 0) {
    writeFileSync(join(outDir, "styles.css"), cssBlocks.join("\n\n"));
    console.log(` Extracted ${cssBlocks.length} CSS block(s) -> styles.css`);
  }

  for (const file of entries) {
    if (file.endsWith(".zippy")) continue;
    const rel = relative(dir, file);
    const outFile = join(outDir, rel);
    const outParent = dirname(outFile);
    if (!existsSync(outParent)) mkdirSync(outParent, { recursive: true });
    copyFileSync(file, outFile);
  }

  const entryPoint = findEntryPoint(dir, outDir);
  if (!entryPoint) {
    console.error(" Could not find entry point. Looked for index.html script tag or App.js/index.js");
    process.exit(1);
  }

  console.log(`\n Bundling ${entryPoint}...`);
  const bundle = spawnSync("bun", ["build", entryPoint, "--outdir", outDir, "--minify"], { stdio: "inherit" });
  if (bundle.status !== 0) {
    console.error(" Bundle failed");
    process.exit(1);
  }

  // Remove intermediate compiled files (subdirectories) — only the bundled output + extracted CSS + html are needed
  console.log("\n Cleaning intermediate files...");
  const subdirs = readdirSync(outDir, { withFileTypes: true })
    .filter(d => d.isDirectory())
    .map(d => join(outDir, d.name));
  for (const dir of subdirs) {
    for (const f of walk(dir)) {
      try { unlinkSync(f); } catch {}
    }
    try { rmdirSync(dir); } catch {}
  }

  console.log("\n Hashing assets...");
  const manifest: Record<string, string> = {};
  const hashTargets = walk(outDir).filter(f => {
    if (f.endsWith(".html")) return false;
    if (f.endsWith(".json")) return false;
    if (f.endsWith(".d.ts")) return false;
    if (f.endsWith(".map")) return false;
    return f.endsWith(".js") || f.endsWith(".css");
  });

  for (const file of hashTargets) {
    const content = readFileSync(file);
    const hash = createHash("sha256").update(content).digest("hex").slice(0, 8);
    const ext = extname(file);
    const base = basename(file, ext);
    const newName = `${base}.${hash}${ext}`;
    const newPath = join(dirname(file), newName);
    if (file !== newPath) {
      renameSync(file, newPath);
    }
    const relPath = relative(outDir, newPath).replace(/\\/g, "/");
    manifest[basename(file)] = relPath;
  }

  writeFileSync(join(outDir, "manifest.json"), JSON.stringify(manifest, null, 2));
  console.log(` Generated manifest.json (${Object.keys(manifest).length} entries)`);

  const htmlPath = join(outDir, "index.html");
  if (existsSync(htmlPath)) {
    let html = readFileSync(htmlPath, "utf-8");

    // Extract original paths from script src and inline imports, then rewrite using manifest
    const originalSrcs = new Set<string>();
    const srcRegex = /<script([^>]*)\s+src="([^"]+)"([^>]*)><\/script>/g;
    for (const m of html.matchAll(srcRegex)) {
      originalSrcs.add(m[2]);
    }
    const importRegex = /import\s+\w+\s+from\s+["']([^"']+)["']/g;
    for (const m of html.matchAll(importRegex)) {
      originalSrcs.add(m[1]);
    }

    for (const orig of originalSrcs) {
      const filename = basename(orig);
      const hashed = manifest[filename];
      if (hashed) {
        html = html.split(orig).join(`./${hashed}`);
      }
    }

    // Ensure CSS links are present in <head>
    const cssEntries = Object.entries(manifest).filter(([k]) => k.endsWith(".css"));
    if (cssEntries.length > 0) {
      const cssLinks = cssEntries
        .map(([, v]) => `<link rel="stylesheet" href="./${v}">`)
        .join("\n  ");
      if (html.includes("</head>")) {
        html = html.replace("</head>", `  ${cssLinks}\n  </head>`);
      } else {
        html = cssLinks + "\n" + html;
      }
    }

    writeFileSync(htmlPath, html);
    console.log(` Rewrote index.html with hashed assets`);
  }

  console.log("\n Build complete!");
}

function findEntryPoint(srcDir: string, outDir: string): string | null {
  const htmlPath = join(srcDir, "index.html");
  if (existsSync(htmlPath)) {
    const html = readFileSync(htmlPath, "utf-8");
    const matches = html.matchAll(/<script[^>]*src="([^"]+)"/g);
    for (const m of matches) {
      let src = m[1];
      if (src.includes("://") || !src.endsWith(".js")) continue;
      const filename = basename(src);
      const distFiles = walk(outDir);
      const found = distFiles.find(f => f.endsWith(filename));
      if (found && existsSync(found)) return found;
    }
  }

  const distFiles = walk(outDir);
  const fallback = distFiles.find(f => f.endsWith("index.js"))
    || distFiles.find(f => f.endsWith("App.js"));
  return fallback || null;
}

function walk(dir: string): string[] {
  const files: string[] = [];
  for (const name of readdirSync(dir)) {
    const full = join(dir, name);
    if (statSync(full).isDirectory()) {
      files.push(...walk(full));
    } else {
      files.push(full);
    }
  }
  return files;
}

function findCompiler(): string | null {
  const dir = import.meta.dir;
  const candidates = [
    join(dir, "..", "..", "compiler", "target", "debug", "zippy-compiler.exe"),
    join(dir, "..", "..", "compiler", "target", "release", "zippy-compiler.exe"),
    "zippy-compiler",
    "zippy-compiler.exe",
  ];
  for (const c of candidates) {
    if (existsSync(c)) return c;
  }
  return null;
}

async function init() {
  console.log(" Creating new Zippy project...");
  console.log(" Done");
}
