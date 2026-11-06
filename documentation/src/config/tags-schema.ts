import { z } from "astro/zod";
import { tagDefinitionSchema } from "starlight-tags";

export const customTagSchema = tagDefinitionSchema.extend({
	// API documentation fields
	type: z.enum(["component", "os", "feature"]).optional(),
	// version: z.string().optional(),
	// apiArea: z.string().optional(),  // e.g., "Security", "Data", "Payments"
	// breaking: z.boolean().optional(),
});

export type CustomTag = z.infer<typeof customTagSchema>;
