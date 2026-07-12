import type { ViteUserConfig } from "astro";
import { fileURLToPath } from "node:url";

import type { starlightLatestVersionConfig } from "..";

export function vitePluginstarlightLatestVersionConfig(
  config: starlightLatestVersionConfig
  // context: starlightLatestVersionContext
): VitePlugin {
  const fetchVersionPath = fileURLToPath(
    new URL("./utils.ts", import.meta.url)
  ).replace(/\\/g, "/");

  const modules = {
    "virtual:starlight-latest-version-config": `export default ${JSON.stringify(
      config
    )}`,
    "virtual:starlight-latest-version": `
import fetchVersion from ${JSON.stringify(fetchVersionPath)};

const config = ${JSON.stringify(config)};

export function getLatestVersion() {
  return fetchVersion(config);
}
`,
  } satisfies Record<string, string>;

  const moduleResolutionMap = Object.fromEntries(
    (Object.keys(modules) as (keyof typeof modules)[]).map((key) => [
      resolveVirtualModuleId(key),
      key,
    ])
  );

  return {
    name: "vite-plugin-starlight-latest-version-config",
    load(id) {
      const moduleId = moduleResolutionMap[id];
      return moduleId ? modules[moduleId] : undefined;
    },
    resolveId(id) {
      return id in modules ? resolveVirtualModuleId(id) : undefined;
    },
  };
}

function resolveVirtualModuleId<TModuleId extends string>(
  id: TModuleId
): `\0${TModuleId}` {
  return `\0${id}`;
}

type VitePlugin = NonNullable<ViteUserConfig["plugins"]>[number];
