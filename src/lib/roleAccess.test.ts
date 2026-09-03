import { describe, expect, it } from "vitest";
import { canAccessPath } from "./roleAccess";

describe("role-based navigation access", () => {
  it("allows administrators and managers to reach all modules", () => {
    expect(canAccessPath("admin", "/settings/users")).toBe(true);
    expect(canAccessPath("manager", "/production/42")).toBe(true);
  });

  it("limits accountants to finance and related operational documents", () => {
    expect(canAccessPath("accountant", "/accounting/statements")).toBe(true);
    expect(canAccessPath("accountant", "/customers/12/statement")).toBe(true);
    expect(canAccessPath("accountant", "/settings/users")).toBe(false);
    expect(canAccessPath("accountant", "/maintenance")).toBe(false);
  });

  it("gives HR and operators focused workspaces", () => {
    expect(canAccessPath("hr", "/payroll")).toBe(true);
    expect(canAccessPath("hr", "/accounting/journal")).toBe(false);
    expect(canAccessPath("operator", "/live-production")).toBe(true);
    expect(canAccessPath("operator", "/cashbank")).toBe(false);
  });

  it("always allows the access-denied and password-change routes", () => {
    expect(canAccessPath("viewer", "/403")).toBe(true);
    expect(canAccessPath("viewer", "/settings/change-password")).toBe(true);
  });
});
