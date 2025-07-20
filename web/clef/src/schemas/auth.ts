import { z } from "zod";

export const LoginFormSchema = z.object({
  username: z.string().min(1, { message: "Username is required" }),
  password: z
    .string()
    .min(1, { message: "Password is required" })
    .min(5, { message: "Password must be at least 5 characters" }),
});

export type LoginForm = z.infer<typeof LoginFormSchema>;

export type LoginResponse = {
  ok: boolean;
  token: string;
};
