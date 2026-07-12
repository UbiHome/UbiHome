import { SEMVER_PATTERN } from "../consts/semantic.version.pattern";
import type { starlightLatestVersionConfig } from "./config";
import type { starlightLatestVersionContext } from "./types";
import { extractVersion, latestReleaseApis } from "./urlBuilder";

export default async function fetchVersion(
  config: starlightLatestVersionConfig
): Promise<starlightLatestVersionContext> {
  const apiUrl = latestReleaseApis[config.source.type](config.source.slug);

  try {
    const data = await fetch(apiUrl).then((response) => {
      if (!response.ok)
        throw new Error(`Failed to fetch: ${response.statusText}`);
      return response.json();
    });

    const tagName = extractVersion[config.source.type](data);
    if (!tagName) {
      return { versionAvailable: false }; // No release available
    }

    const match = tagName.match(config.regexPattern ?? SEMVER_PATTERN);

    if (!match) {
      return { versionAvailable: false }; // No valid version found
    }

    const versionWithoutPrefix = match.groups?.version || "";
    const [versionMajor = 0, versionMinor = 0, versionPatch = 0] =
      versionWithoutPrefix.split(".").map(Number);

    const prerelease = match.groups?.prerelease;
    const isPrereleaseVersion = !!prerelease;
    const version = isPrereleaseVersion
      ? `v${versionWithoutPrefix}-${prerelease}`
      : `v${versionWithoutPrefix}`;

    const prefixMatch = tagName.match(/^(.*?)v?[0-9]/);
    const prefix = prefixMatch ? prefixMatch[1] : undefined;
    const hasVPrefix = tagName.startsWith("v") || tagName.includes("@v");

    const context: starlightLatestVersionContext = {
      versionAvailable: true,
      version,
      versionWithoutPrefix,
      versionPatch,
      versionMinor,
      versionMajor,
      prerelease,
      isPrereleaseVersion,
      prefix,
      hasVPrefix,
      isStableVersion: !isPrereleaseVersion,
    };

    return context;
  } catch (error) {
    console.error(error);
    return { versionAvailable: false }; // Fallback: no version available
  }
}
