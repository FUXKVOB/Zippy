import { serve, file } from "bun";
import { watch, existsSync, mkdtempSync, rmSync, mkdirSync, readFileSync } from "fs";
import { join, extname, dirname, resolve } from "path";
import { execSync } from "child_process";
import { tmpdir } from "os";

const ROOT = join(import.meta.dir, "..", "..", "..");
const COMPILER = join(ROOT, "packages", "compiler", "target", "debug", "zippy-compiler.exe");
const PORT = parseInt(process.env.PORT || "3000");

function ensureWorkspaceLinks() {
  const runtimeLink = join(ROOT, "node_modules", "@zippy", "runtime");
  if (!existsSync(runtimeLink)) {
    const scopeDir = join(ROOT, "node_modules", "@zippy");
    if (!existsSync(scopeDir)) mkdirSync(scopeDir, { recursive: true });
    try {
      execSync(`cmd /c mklink /J "${runtimeLink}" "${join(ROOT, "packages", "runtime")}"`, { stdio: "pipe" });
      console.log("  Created @zippy/runtime workspace link");
    } catch {
      console.warn("  Warning: could not create workspace link for @zippy/runtime");
    }
  }
}
ensureWorkspaceLinks();

console.log(`⚡ Zippy dev server`);
console.log(`  Root: ${ROOT}`);
console.log(`  URL:  http://localhost:${PORT}`);

let bundleCache = new Map<string, { data: string; mtime: number }>();
let sockets = new Set<Bun.ServerWebSocket>();

// Compile a .zippy file to .js if needed (returns true if compiled)
function ensureCompiled(zippyPath: string): boolean {
  if (!existsSync(zippyPath)) return false;
  const jsPath = zippyPath.replace(/\.zippy$/, ".js");
  // If .js doesn't exist, or .zippy is newer, recompile
  if (!existsSync(jsPath)) {
    try {
      execSync(`"${COMPILER}" "${zippyPath}" "${jsPath}"`, { stdio: "pipe", timeout: 10000 });
      bundleCache.delete(jsPath);
      return true;
    } catch (e) {
      console.error(`  Compile error: ${zippyPath}`, (e as Error).message);
      return false;
    }
  }
  return true;
}

// Find all .zippy files imported in a .js file (recursively)
function compileAllDependencies(jsPath: string, visited = new Set<string>()): void {
  if (visited.has(jsPath)) return;
  visited.add(jsPath);

  if (!existsSync(jsPath)) return;
  const content = readFileSync(jsPath, "utf-8");

  // Match import statements: import X from "./Foo.js" or "./Foo.zippy"
  const importRegex = /import\s+[\w*\s{},]+\s+from\s+["']([^"']+)["']/g;
  let match;
  while ((match = importRegex.exec(content)) !== null) {
    const importPath = match[1];
    if (!importPath.startsWith(".")) continue; // skip bare imports like @zippy/runtime
    const dir = dirname(jsPath);
    const resolved = resolve(dir, importPath);
    
    // Check for .zippy source (most common case)
    if (resolved.endsWith(".zippy")) {
      if (ensureCompiled(resolved)) {
        compileAllDependencies(resolved.replace(/\.zippy$/, ".js"), visited);
      }
    } else if (resolved.endsWith(".js")) {
      // Check if the .js exists; if not, try compiling from .zippy
      if (!existsSync(resolved)) {
        const zippyPath = resolved.replace(/\.js$/, ".zippy");
        if (existsSync(zippyPath)) {
          ensureCompiled(zippyPath);
        }
      }
      // Recurse into .js files to find their dependencies
      if (existsSync(resolved)) {
        compileAllDependencies(resolved, visited);
      }
    }
  }
}

watch(ROOT, { recursive: true }, (ev, filename) => {
  if (!filename?.endsWith(".zippy")) return;
  const input = join(ROOT, filename);
  const output = input.replace(/\.zippy$/, ".js");
  try {
    execSync(`"${COMPILER}" "${input}" "${output}"`, { stdio: "pipe", timeout: 10000 });
    bundleCache.delete(output);
    console.log(`  Recompiled ${filename}`);
    
    // Notify clients about the update
    const url = new URL(output).pathname; // Simplified
    for (const ws of sockets) {
      ws.send(JSON.stringify({ type: 'update', file: filename }));
    }
  } catch (e) {
    console.error(`  Error: ${filename}`, (e as Error).message);
  }
});

const ERROR_OVERLAY_SCRIPT = `
<script>
(function() {
  function showOverlay(msg, stack) {
    let el = document.getElementById('zippy-error-overlay');
    if (!el) {
      el = document.createElement('div');
      el.id = 'zippy-error-overlay';
      el.style.cssText = 'position:fixed;top:0;left:0;right:0;bottom:0;background:rgba(0,0,0,0.85);color:#fff;font-family:monospace;padding:24px;z-index:99999;overflow:auto;';
      document.body.appendChild(el);
    }
    el.innerHTML = '<div style="background:#1a1a2e;border:1px solid #e34c4c;border-radius:8px;padding:20px;max-width:900px;margin:0 auto;"><h2 style="color:#e34c4c;margin:0 0 12px;">⚡ Zippy Runtime Error</h2><pre style="white-space:pre-wrap;color:#ff6b6b;margin:0 0 16px;">' + msg + '</pre><pre style="white-space:pre-wrap;color:#888;font-size:12px;margin:0;">' + (stack || '') + '</pre><button onclick="document.getElementById(\\'zippy-error-overlay\\').remove()" style="margin-top:16px;padding:8px 16px;background:#e34c4c;color:#fff;border:none;border-radius:4px;cursor:pointer;">Dismiss</button></div>';
  }
  window.addEventListener('error', function(e) {
    showOverlay(e.message || 'Unknown error', e.error && e.error.stack);
  });
  window.addEventListener('unhandledrejection', function(e) {
    showOverlay('Unhandled Promise rejection: ' + (e.reason && e.reason.message || e.reason), e.reason && e.reason.stack);
  });
  if (window.__ZIPPY_HMR__) {
    window.__ZIPPY_HMR__.onError = function(err) { showOverlay(err, ''); };
  }
})();
</script>
`;

serve({
  port: PORT,
  async fetch(req, server) {
    // Upgrade to WebSocket if requested
    if (server.upgrade(req)) return;

    const url = new URL(req.url);
    let filePath = join(ROOT, url.pathname);

    // SPA fallback: if path has no extension and no file exists, serve index.html
    if (!extname(filePath) && !existsSync(filePath)) {
      // Try to find index.html in the same directory
      const indexPath = join(filePath, "index.html");
      if (existsSync(indexPath)) {
        filePath = indexPath;
      } else {
        // Fall back to root index.html for SPA routing
        const rootIndex = join(ROOT, "index.html");
        if (existsSync(rootIndex)) {
          filePath = rootIndex;
        }
      }
    } else if (!extname(filePath)) {
      filePath = join(filePath, "index.html");
    }

    if (filePath.endsWith(".js") && !existsSync(filePath)) {
      // Try to compile from .zippy source on demand
      const zippyPath = filePath.replace(/\.js$/, ".zippy");
      if (existsSync(zippyPath)) {
        ensureCompiled(zippyPath);
      }
    }

    if (filePath.endsWith(".js") && existsSync(filePath)) {
      // Recursively compile all .zippy dependencies
      compileAllDependencies(filePath);
      const src = Bun.file(filePath);
      const mtime = (await src.stat()).mtimeMs;
      const cached = bundleCache.get(filePath);
      if (cached && cached.mtime === mtime) {
        return new Response(cached.data, {
          headers: { "Content-Type": "application/javascript" },
        });
      }

      const tmp = mkdtempSync(join(tmpdir(), "zippy-"));
      try {
        const result = await Bun.build({
          entrypoints: [filePath],
          outdir: tmp,
          format: "esm",
          sourcemap: "inline",
        });

        if (!result.success) {
          return new Response(`// Build error`, {
            status: 500,
            headers: { "Content-Type": "application/javascript" },
          });
        }

        const outFile = result.outputs[0].path;
        const data = await Bun.file(outFile).text();
        bundleCache.set(filePath, { data, mtime });
        return new Response(data, {
          headers: { "Content-Type": "application/javascript" },
        });
      } finally {
        try { rmSync(tmp, { recursive: true }); } catch {}
      }
    }

    // Serve index.html with error overlay injected
    if (filePath.endsWith("index.html") && existsSync(filePath)) {
      const html = Bun.file(filePath);
      const content = await html.text();
      const injected = content.replace("</head>", ERROR_OVERLAY_SCRIPT + "</head>");
      return new Response(injected, { headers: { "Content-Type": "text/html" } });
    }

    const f = file(filePath);
    if (f.size <= 0) return new Response("Not Found", { status: 404 });
    return new Response(f);
  },
  websocket: {
    open(ws) {
      sockets.add(ws);
    },
    close(ws) {
      sockets.delete(ws);
    },
    message(ws, msg) {
      // No-op
    }
  }
});

console.log(`  Ready.`);
