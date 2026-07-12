import { z } from "astro/zod";

// Enum-like structure for the version state
export const VersionState = z.union([
  z.object({
    versionAvailable: z.literal(true),
    version: z.string(),
    versionWithoutPrefix: z.string(),
    versionMajor: z.number(),
    versionMinor: z.number(),
    versionPatch: z.number(),
    prerelease: z.string().optional(),
    isPrereleaseVersion: z.boolean(),
    prefix: z.string().optional(),
    hasVPrefix: z.boolean(),
    isStableVersion: z.boolean(),
  }),
  z.object({
    versionAvailable: z.literal(false),
  }),
]);

export type starlightLatestVersionContext = z.infer<typeof VersionState>;
