import type { StarlightPlugin } from "@astrojs/starlight/types";

import {
  type starlightLatestVersionConfig,
  type starlightLatestVersionUserConfig,
  validateConfig,
} from "./libs/config";
import { overrideStarlightComponent } from "./libs/starlight";
import type { starlightLatestVersionContext } from "./libs/types";
import { vitePluginstarlightLatestVersionConfig } from "./libs/vite";

export type {
  starlightLatestVersionConfig,
  starlightLatestVersionContext,
  starlightLatestVersionUserConfig,
};

export default function starlightLatestVersion(
  userConfig?: starlightLatestVersionUserConfig
): StarlightPlugin {
  const config = validateConfig(userConfig);

  return {
    name: "starlight-latest-version",
    hooks: {
      "config:setup"({
        addIntegration,
        updateConfig: updateStarlightConfig,
        config: starlightConfig,
        logger,
      }) {
        updateStarlightConfig({
          components: {
            ...starlightConfig.components,
            ...(config.showInSiteTitle !== "false"
              ? overrideStarlightComponent(
                  starlightConfig.components,
                  logger,
                  "SiteTitle",
                  "DynamicVersionBadge"
                )
              : {}),
          },
        });

        addIntegration({
          name: "starlight-latest-version-integration",
          hooks: {
            "astro:config:setup": ({ updateConfig }) => {
              updateConfig({
                vite: {
                  plugins: [vitePluginstarlightLatestVersionConfig(config)],
                },
              });
            },
            "astro:config:done": ({ injectTypes }) => {
              injectTypes({
                filename: "types.d.ts",
                content: `declare module "virtual:starlight-latest-version" {
  export function getLatestVersion(): Promise<
    import("starlight-latest-version").starlightLatestVersionContext
  >;
}`,
              });
            },
          },
        });
      },
    },
  };
}
