import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { formatOMR, formatNumber, getStatusColor, getSeverityColor, debounce } from "./utils";

describe("formatOMR", () => {
  it("converts milli to OMR with 3 decimals", () => {
    expect(formatOMR(1000)).toBe("1.000 ر.ع");
    expect(formatOMR(105_000)).toBe("105.000 ر.ع");
  });

  it("handles zero and fractional milli", () => {
    expect(formatOMR(0)).toBe("0.000 ر.ع");
    expect(formatOMR(1)).toBe("0.001 ر.ع");
  });
});

describe("formatNumber", () => {
  it("formats with en-US thousands separators", () => {
    expect(formatNumber(1234)).toBe("1,234");
    expect(formatNumber(0)).toBe("0");
  });
});

describe("getStatusColor", () => {
  it("maps success statuses", () => {
    expect(getStatusColor("posted")).toBe("badge-success");
    expect(getStatusColor("PAID")).toBe("badge-success");
  });

  it("maps danger statuses", () => {
    expect(getStatusColor("void")).toBe("badge-danger");
    expect(getStatusColor("cancelled")).toBe("badge-danger");
  });

  it("maps warning statuses", () => {
    expect(getStatusColor("pending")).toBe("badge-warning");
    expect(getStatusColor("submitted")).toBe("badge-warning");
  });

  it("defaults to info", () => {
    expect(getStatusColor("unknown")).toBe("badge-info");
  });
});

describe("getSeverityColor", () => {
  it("maps high and critical to danger", () => {
    expect(getSeverityColor("high")).toBe("badge-danger");
    expect(getSeverityColor("critical")).toBe("badge-danger");
  });

  it("maps medium to warning and low to success", () => {
    expect(getSeverityColor("medium")).toBe("badge-warning");
    expect(getSeverityColor("low")).toBe("badge-success");
  });
});

describe("debounce", () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it("calls the function only after the delay", () => {
    const fn = vi.fn();
    const debounced = debounce(fn, 300);

    debounced();
    expect(fn).not.toHaveBeenCalled();

    vi.advanceTimersByTime(299);
    expect(fn).not.toHaveBeenCalled();

    vi.advanceTimersByTime(1);
    expect(fn).toHaveBeenCalledTimes(1);
  });

  it("resets the timer on repeated calls", () => {
    const fn = vi.fn();
    const debounced = debounce(fn, 300);

    debounced();
    vi.advanceTimersByTime(100);
    debounced();
    vi.advanceTimersByTime(299);
    expect(fn).not.toHaveBeenCalled();

    vi.advanceTimersByTime(1);
    expect(fn).toHaveBeenCalledTimes(1);
  });
});
