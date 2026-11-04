import { describe, expect, test } from "bun:test";
import { PassThrough } from "node:stream";
import {
  WORK_PERSPECTIVES,
  makeSelection,
  moveSelection,
  parseWorkPerspective,
} from "../src/work-perspective.js";
import { parseCommand } from "../src/cli.js";
import { promptWorkPerspective } from "../src/selector.js";

describe("work perspective selection", () => {
  test("offers exactly the three approved choices with explanations", () => {
    expect(WORK_PERSPECTIVES.map((choice) => choice.value)).toEqual([
      "product_engineer",
      "fde",
      "mixed",
    ]);
    expect(WORK_PERSPECTIVES.every((choice) => choice.description.length > 0)).toBe(true);
  });

  test("requires one of the explicit choices", () => {
    expect(parseWorkPerspective("product_engineer")).toBe("product_engineer");
    expect(parseWorkPerspective("fde")).toBe("fde");
    expect(parseWorkPerspective("mixed")).toBe("mixed");
    expect(() => parseWorkPerspective("auto")).toThrow("must be product_engineer, fde, or mixed");
    expect(() => parseWorkPerspective("other")).toThrow("must be product_engineer, fde, or mixed");
  });

  test("cycles through the selector choices", () => {
    expect(moveSelection(0, -1)).toBe(2);
    expect(moveSelection(2, 1)).toBe(0);
    expect(moveSelection(0, 1)).toBe(1);
  });

  test("returns a versioned selection result without claiming a Mission was created", () => {
    expect(makeSelection("fde", "explicit_argument")).toEqual({
      schema: "tramuri.work-perspective-selection.v1",
      work_perspective: "fde",
      selection_method: "explicit_argument",
      mission_created: false,
    });
  });

  test("supports explicit selection for non-interactive callers", () => {
    expect(parseCommand(["perspective", "choose", "--perspective", "mixed", "--json"])).toEqual({
      help: false,
      json: true,
      perspective: "mixed",
    });
    expect(() => parseCommand(["perspective", "choose", "--perspective"])).toThrow("requires a value");
  });

  test("interactive selector returns a choice only after arrow navigation and Enter", async () => {
    const input = new PassThrough() as PassThrough & {
      isTTY: boolean;
      setRawMode: (enabled: boolean) => void;
    };
    input.isTTY = true;
    const rawModeChanges: boolean[] = [];
    input.setRawMode = (enabled) => rawModeChanges.push(enabled);

    let screen = "";
    const output = {
      isTTY: true,
      write: (value: string) => {
        screen += value;
      },
    };

    const pending = promptWorkPerspective(input, output);
    input.write("\u001b[B\r");

    await expect(pending).resolves.toBe("fde");
    expect(rawModeChanges).toEqual([true, false]);
    expect(screen).toContain("Necessidade do cliente, sistemas, restrições e aceite.");
  });
});
