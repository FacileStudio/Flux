#!/usr/bin/env bun
import { $ } from "bun";

type Tool = { name: string; repo: string; bin: string; bins?: string[]; asset?: string };
type Asset = { name: string; url: string; digest?: string };
type Entry = { version: string; repo: string; bins: string[]; systems: Record<string, { url: string; hash: string }> };

const skip = new Set(["widgets"]);
const arches: Record<string, string> = { "x86_64-linux": "amd64", "aarch64-linux": "arm64" };
const out = new URL("../pkgs/facile-tools/tools.json", import.meta.url).pathname;

/** Reads the catalog from a local path when given, else from facile's main branch. */
async function loadCatalog(path?: string): Promise<Tool[]> {
  const text = path
    ? await Bun.file(path).text()
    : await $`gh api repos/FacileStudio/facile/contents/internal/manifest/tools.yml -H ${"Accept: application/vnd.github.raw"}`.text();
  return (Bun.YAML.parse(text) as { tools: Tool[] }).tools;
}

/** Returns the SRI hash of an asset, from GitHub's digest or by downloading it. */
async function sri(asset: Asset): Promise<string> {
  const hex = asset.digest?.startsWith("sha256:")
    ? asset.digest.slice(7)
    : new Bun.CryptoHasher("sha256").update(await (await fetch(asset.url)).arrayBuffer()).digest("hex");
  return `sha256-${Buffer.from(hex, "hex").toString("base64")}`;
}

/** Resolves one tool's latest release into a lock entry, or a reason it was left out. */
async function lock(tool: Tool): Promise<Entry | string> {
  if (!tool.asset) return "no release asset";
  const visibility = (await $`gh repo view ${tool.repo} --json visibility -q .visibility`.nothrow().text()).trim();
  if (visibility !== "PUBLIC") return `repo is ${visibility || "unreachable"}`;
  const view = await $`gh release view -R ${tool.repo} --json tagName,assets`.nothrow().quiet();
  if (view.exitCode !== 0) return "no release";
  const release = view.json() as { tagName: string; assets: Asset[] };
  const version = release.tagName.replace(/^v/, "");
  const systems: Entry["systems"] = {};
  for (const [system, arch] of Object.entries(arches)) {
    const asset = release.assets.find((a) => a.name === `${tool.asset}_${version}_linux_${arch}.tar.gz`);
    if (asset) systems[system] = { url: asset.url, hash: await sri(asset) };
  }
  if (Object.keys(systems).length === 0) return "no linux asset";
  return { version, repo: tool.repo, bins: tool.bins ?? [tool.bin], systems };
}

const tools = (await loadCatalog(Bun.argv[2])).filter((t) => !skip.has(t.name));
const results = await Promise.all(tools.map(async (t) => [t.name, await lock(t)] as const));
const locked: Record<string, Entry> = {};
for (const [name, result] of results.sort(([a], [b]) => a.localeCompare(b))) {
  if (typeof result === "string") console.error(`skip ${name}: ${result}`);
  else {
    locked[name] = result;
    const missing = Object.keys(arches).filter((s) => !result.systems[s]);
    if (missing.length) console.error(`note ${name}: no asset for ${missing.join(", ")}`);
  }
}
await Bun.write(out, JSON.stringify(locked, null, 2) + "\n");
console.error(`wrote ${Object.keys(locked).length} tools to ${out}`);
