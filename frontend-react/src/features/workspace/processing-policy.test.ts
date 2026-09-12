import { describe, expect, it } from "vitest";
import {
  effectiveProcessingLocation,
  validateDeviceProcessingSize,
} from "@/features/workspace/processing-policy";
import type { ToolDef } from "@/lib/tools-catalog";

const baseTool = (overrides: Partial<ToolDef> = {}): ToolDef => ({
  slug: "watermark",
  title: "Watermark",
  short: "Watermark",
  description: "",
  hint: "",
  group: "atur",
  accept: "pdf",
  multiple: false,
  minFiles: 1,
  processing: "device",
  ...overrides,
});

const options = (splitEach = false) => ({ splitEach });

const makeFile = (size: number) => ({ size, name: "sample.pdf" }) as File;

describe("processing policy", () => {
  it("keeps the declared server execution mode", () => {
    expect(
      effectiveProcessingLocation(
        baseTool({ processing: "server" }),
        options(),
      ),
    ).toBe("server");
  });

  it("uses device execution for per-page split", () => {
    expect(
      effectiveProcessingLocation(
        baseTool({ slug: "pisah", processing: "server" }),
        options(true),
      ),
    ).toBe("device");
  });

  it("rejects oversized device-side input", () => {
    expect(() =>
      validateDeviceProcessingSize(
        baseTool({ processing: "device" }),
        [makeFile(100 * 1024 * 1024 + 1)],
        options(),
      ),
    ).toThrow("100 MiB");
  });

  it("allows oversized server-side input", () => {
    expect(() =>
      validateDeviceProcessingSize(
        baseTool({ processing: "server" }),
        [makeFile(100 * 1024 * 1024 + 1)],
        options(),
      ),
    ).not.toThrow();
  });
});
