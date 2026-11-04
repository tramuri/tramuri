import { emitKeypressEvents } from "node:readline";
import { WORK_PERSPECTIVES, type WorkPerspective } from "./work-perspective.js";

interface KeyEvent {
  name?: string;
  ctrl?: boolean;
}

interface RawInput {
  isTTY?: boolean;
  setRawMode?: (enabled: boolean) => void;
  resume: () => void;
  pause: () => void;
  on: (event: "keypress", listener: (text: string, key: KeyEvent) => void) => void;
  off: (event: "keypress", listener: (text: string, key: KeyEvent) => void) => void;
}

interface TerminalOutput {
  isTTY?: boolean;
  write: (value: string) => void;
}

function renderMenu(selected: number, color: boolean): string[] {
  const teal = color ? "\u001b[38;2;0;179;166m" : "";
  const reset = color ? "\u001b[0m" : "";
  const rows = [
    "Tramuri · escolha a perspectiva desta Mission",
    "Use ↑/↓ para navegar, Enter para escolher ou Esc para cancelar.",
    "",
  ];

  WORK_PERSPECTIVES.forEach((choice, index) => {
    const marker = index === selected ? `${teal}›${reset}` : " ";
    const label = index === selected && color
      ? `${teal}${choice.label}${reset}`
      : choice.label;
    rows.push(`${marker} ${label}`);
    rows.push(`  ${choice.description}`);
  });

  return rows;
}

export async function promptWorkPerspective(
  input = process.stdin as unknown as RawInput,
  output = process.stderr as unknown as TerminalOutput,
): Promise<WorkPerspective> {
  if (!input.isTTY || typeof input.setRawMode !== "function" || !output.isTTY) {
    throw new Error(
      "interactive perspective selection requires a terminal; pass --perspective product_engineer, --perspective fde, or --perspective mixed",
    );
  }

  emitKeypressEvents(input as never);
  input.setRawMode(true);
  input.resume();

  const color = !process.env.NO_COLOR;
  let selected = 0;
  let renderedRows = 0;

  const draw = (): void => {
    const rows = renderMenu(selected, color);
    if (renderedRows > 0) output.write(`\u001b[${renderedRows}F`);
    for (const row of rows) output.write(`\u001b[2K\r${row}\n`);
    renderedRows = rows.length;
  };

  return await new Promise<WorkPerspective>((resolve, reject) => {
    const restoreTerminal = (): void => {
      input.off("keypress", onKeypress);
      input.setRawMode?.(false);
      input.pause();
      output.write("\u001b[?25h\n");
    };

    const onKeypress = (_text: string, key: KeyEvent): void => {
      if (key.ctrl && key.name === "c") {
        restoreTerminal();
        reject(new Error("perspective selection cancelled"));
        return;
      }

      if (key.name === "escape") {
        restoreTerminal();
        reject(new Error("perspective selection cancelled"));
        return;
      }

      if (key.name === "up") {
        selected = (selected + WORK_PERSPECTIVES.length - 1) % WORK_PERSPECTIVES.length;
        draw();
      } else if (key.name === "down") {
        selected = (selected + 1) % WORK_PERSPECTIVES.length;
        draw();
      } else if (key.name === "return") {
        const choice = WORK_PERSPECTIVES[selected];
        if (!choice) {
          restoreTerminal();
          reject(new Error("no perspective is selected"));
          return;
        }
        restoreTerminal();
        resolve(choice.value);
      }
    };

    output.write("\u001b[?25l");
    draw();
    input.on("keypress", onKeypress);
  });
}
