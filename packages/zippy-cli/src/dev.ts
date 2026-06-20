import { serve, file } from "bun";
import { watch } from "fs";
import { join, extname } from "path";
import { execSync } from "child_process";

const ROOT = join(import.meta.dir, "..", "..", "..");
const COMPILER = join(ROOT, "packages", "compiler", "target", "debug", "zippy-compiler.exe");
const PORT = parseInt(process.env.PORT || "3000");

console.log(`⚡ Zippy dev server`);
console.log(`  Root: ${ROOT}`);
console.log(`  URL:  http://localhost:${PORT}`);

// Watch .zippy files
const watcher = watch(ROOT, { recursive: true }, (ev, filename) => {
  if (!filename?.endsWith(".zippy")) return;
  const input = join(ROOT, filename);
  const output = input.replace(/\.zippy$/, ".js");
  try {
    execSync(`"${COMPILER}" "${input}" "${output}"`, { stdio: "pipe", timeout: 10000 });
    console.log(`  Recompiled ${filename}`);
  } catch (e) {
    console.error(`  Error: ${filename}`, e.message);
  }
});

// HTTP server
serve({
  port: PORT,
  fetch(req) {
    const url = new URL(req.url);
    let filePath = join(ROOT, url.pathname);

    if (!extname(filePath)) filePath = join(filePath, "index.html");

    const f = file(filePath);
    const exists = f.size > 0;
    if (!exists) return new Response("Not Found", { status: 404 });

    return new Response(f);
  },
});

console.log(`  Ready.`);
