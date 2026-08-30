import { describe, expect, it } from "vitest";

type PdfTextItem = {
  transform: number[];
  str: string;
};

function isPdfTextItem(item: unknown): item is PdfTextItem {
  if (!item || typeof item !== "object") {
    return false;
  }

  const candidate = item as {
    transform?: unknown;
    str?: unknown;
  };

  return (
    Array.isArray(candidate.transform) &&
    candidate.transform.length >= 6 &&
    candidate.transform.every(
      (value) =>
        typeof value === "number" && Number.isFinite(value),
    ) &&
    typeof candidate.str === "string"
  );
}

describe("PDF text item guard", () => {
  it("accepts valid text items", () => {
    expect(
      isPdfTextItem({
        transform: [1, 0, 0, 1, 10, 20],
        str: "hello",
      }),
    ).toBe(true);
  });

  it("rejects marked-content and malformed items", () => {
    expect(isPdfTextItem({ type: "markedContent", id: 1 })).toBe(false);
    expect(
      isPdfTextItem({
        transform: [1, 0, 0],
        str: "hello",
      }),
    ).toBe(false);
    expect(
      isPdfTextItem({
        transform: [1, 0, 0, 1, 10, Number.NaN],
        str: "hello",
      }),
    ).toBe(false);
  });
});
