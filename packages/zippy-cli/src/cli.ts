#!/usr/bin/env bun

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
  console.log(` Building ${dir}...`);
  // TODO: traverse dir, compile .zippy files, bundle output
  console.log(" Done");
}

async function init() {
  console.log(" Creating new Zippy project...");
  // TODO: scaffold template project
  console.log(" Done");
}
