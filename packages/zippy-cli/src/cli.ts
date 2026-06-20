#!/usr/bin/env bun

import { spawnSync } from "child_process";
import { join, relative, dirname } from "path";
import { readdirSync, existsSync, mkdirSync, statSync, copyFileSync } from "fs";

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

  // Copy non-zippy assets
  for (const file of entries) {
    if (file.endsWith(".zippy")) continue;
    const rel = relative(dir, file);
    const outFile = join(outDir, rel);
    const outParent = dirname(outFile);
    if (!existsSync(outParent)) mkdirSync(outParent, { recursive: true });
    copyFileSync(file, outFile);
  }

  // Bundle with bun
  console.log("\n Bundling...");
  const bundle = spawnSync("bun", ["build", `./${outDir}/index.js`, "--outdir", outDir, "--minify"], { stdio: "inherit" });
  if (bundle.status !== 0) {
    console.error(" Bundle failed");
    process.exit(1);
  }

  console.log("\n Build complete!");
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
