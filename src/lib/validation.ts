import i18n from "@/i18n";

export type Validator = (value: unknown) => string | null;

export const required = (msg?: string): Validator => (value) => {
  const message = msg ?? i18n.t("validation.required");
  if (typeof value === "string") return value.trim() ? null : message;
  if (typeof value === "number") return Number.isFinite(value) ? null : message;
  return message;
};

export const requiredPick = (msg?: string): Validator => (value) => {
  const message = msg ?? i18n.t("validation.requiredPick");
  if (typeof value === "number") return value > 0 ? null : message;
  if (typeof value === "string") return value.trim() && value.trim() !== "0" ? null : message;
  return message;
};

export const nonNegative = (msg?: string): Validator => (value) => {
  const message = msg ?? i18n.t("validation.nonNegative");
  if (typeof value === "number") return Number.isFinite(value) && value >= 0 ? null : message;
  if (typeof value === "string" && value.trim() === "") return null;
  const n = Number(value);
  return Number.isFinite(n) && n >= 0 ? null : message;
};

export const positive = (msg?: string): Validator => (value) => {
  const message = msg ?? i18n.t("validation.positive");
  if (typeof value === "string" && value.trim() === "") return null;
  const n = Number(value);
  return Number.isFinite(n) && n > 0 ? null : message;
};

export const email = (msg?: string): Validator => (value) => {
  const message = msg ?? i18n.t("validation.email");
  if (typeof value !== "string" || !value.trim()) return null;
  return /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(value.trim()) ? null : message;
};

export const phone = (msg?: string): Validator => (value) => {
  const message = msg ?? i18n.t("validation.phone");
  if (typeof value !== "string" || !value.trim()) return null;
  return /^[0-9+\-\s]{7,15}$/.test(value.trim()) ? null : message;
};

export type ValidationSpec = Record<string, Validator[]>;

export function validate(values: Record<string, unknown>, spec: ValidationSpec): Record<string, string> {
  const errors: Record<string, string> = {};
  for (const [key, validators] of Object.entries(spec)) {
    for (const validator of validators) {
      const message = validator(values[key]);
      if (message) {
        errors[key] = message;
        break;
      }
    }
  }
  return errors;
}

export function hasErrors(errors: Record<string, string>): boolean {
  return Object.keys(errors).length > 0;
}

export function clearError(errors: Record<string, string>, key: string): Record<string, string> {
  if (!(key in errors)) return errors;
  const next = { ...errors };
  delete next[key];
  return next;
}
