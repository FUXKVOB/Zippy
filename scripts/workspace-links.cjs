const { existsSync, mkdirSync } = require("fs");
const { execSync } = require("child_process");
const { join } = require("path");

const root = __dirname;
const link = join(root, "..", "node_modules", "@zippy", "runtime");
if (!existsSync(link)) {
  const scopeDir = join(root, "..", "node_modules", "@zippy");
  if (!existsSync(scopeDir)) mkdirSync(scopeDir, { recursive: true });
  const target = join(root, "..", "packages", "runtime");
  execSync(`cmd /c mklink /J "${link}" "${target}"`, { stdio: "pipe" });
  console.log("Created @zippy/runtime workspace link");
}
