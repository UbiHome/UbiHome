export const releasePageUrls: Record<
  "github" | "gitlab" | "npm",
  (slug: string) => string
> = {
  github: (slug) => `https://github.com/${slug}/releases`,
  gitlab: (slug) => `https://gitlab.com/${slug}/-/releases`,
  npm: (slug) => `https://www.npmjs.com/package/${slug}?activeTab=versions`,
};

export const latestReleaseApis: Record<
  "github" | "gitlab" | "npm",
  (slug: string) => string
> = {
  github: (slug) => `https://api.github.com/repos/${slug}/releases/latest`,
  gitlab: (slug) =>
    `https://gitlab.com/api/v4/projects/${encodeURIComponent(slug)}/releases`,
  npm: (slug) => `https://registry.npmjs.org/${slug}/latest`,
};

export const extractVersion: Record<
  "github" | "gitlab" | "npm",
  (data: any) => string | null
> = {
  github: (data) => data?.tag_name || null,
  gitlab: (data) => data?.[0]?.tag_name || null,
  npm: (data) => data?.version || null,
};
