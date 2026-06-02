import { cpSync, mkdirSync, rmSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");

for (const browser of ["chrome", "edge"]) {
  const output = join(root, "packages", browser);
  rmSync(output, { recursive: true, force: true });
  mkdirSync(join(output, "dist"), { recursive: true });
  mkdirSync(join(output, "rules"), { recursive: true });
  cpSync(join(root, "manifest.json"), join(output, "manifest.json"));
  cpSync(join(root, "popup.html"), join(output, "popup.html"));
  cpSync(join(root, "dist"), join(output, "dist"), { recursive: true });
  cpSync(join(root, "rules"), join(output, "rules"), { recursive: true });
}

