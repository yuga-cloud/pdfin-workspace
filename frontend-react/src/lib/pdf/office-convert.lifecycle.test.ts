import { describe, expect, it } from "vitest";
import { isPdfTextItem } from "./office-convert";

describe("PDF text item guard", () => {
  it("accepts valid text items", () => {
    expect(
      isPdfTextItem({
        transform: [1, 0, 0, 1, 10, 20],
        str: "hello",
      }),
    ).toBe(true);
  });

  it("rejects malformed and non-text items", () => {
    expect(
      isPdfTextItem({
        type: "markedContent",
        id: 1,
      }),
    ).toBe(false);
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
