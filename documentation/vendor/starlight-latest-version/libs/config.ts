import { AstroError } from "astro/errors";
import { z } from "astro/zod";

const configSchema = z.object({
  source: z.object({
    type: z.enum(["github", "gitlab", "npm"]).default("npm"),
    slug: z.string().min(1, "Slug cannot be empty"),
  }),
  badge: z
    .object({
      variant: z
        .enum(["default", "note", "danger", "success", "caution", "tip"])
        .default("default"),
      size: z.enum(["small", "medium", "large"]).default("medium"),
    })
    .prefault({}),
  showInSiteTitle: z.enum(["false", "true", "deferred"]).default("false"),
  regexPattern: z.string().optional(),
});

export function validateConfig(
  userConfig: unknown
): starlightLatestVersionConfig {
  const config = configSchema.safeParse(userConfig);

  if (!config.success) {
    const errors = config.error.flatten();

    throw new AstroError(
      `Invalid starlight-latest-version configuration:

      ${errors.formErrors.map((formError) => ` - ${formError}`).join("\n")}
      ${Object.entries(errors.fieldErrors)
        .map(
          ([fieldName, fieldErrors]) =>
            ` - ${fieldName}: ${fieldErrors.join(" - ")}`
        )
        .join("\n")}
        `,
      `See the error report above for more informations.\n\nIf you believe this is a bug, please file an issue at https://github.com/trueberryless-org/starlight-latest-version/issues/new`
    );
  }

  return config.data;
}

export type starlightLatestVersionUserConfig = z.input<typeof configSchema>;
export type starlightLatestVersionConfig = z.output<typeof configSchema>;
