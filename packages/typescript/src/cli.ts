#!/usr/bin/env node

import { fileURLToPath } from "node:url";
import { resolve } from "node:path";
import {
  makeSelection,
  parseWorkPerspective,
  type SelectionMethod,
  type WorkPerspective,
} from "./work-perspective.js";
import { promptWorkPerspective } from "./selector.js";

interface ParsedCommand {
  help: boolean;
  json: boolean;
  perspective?: WorkPerspective;
}

export function parseCommand(args: string[]): ParsedCommand {
  if (args[0] === "--help" || args[0] === "-h") {
    return { help: true, json: false };
  }
  if (args[0] !== "perspective" || args[1] !== "choose") {
    throw new Error("expected command: perspective choose");
  }

  let json = false;
  let perspective: WorkPerspective | undefined;
  for (let index = 2; index < args.length; index += 1) {
    const argument = args[index];
    if (argument === "--json") {
      json = true;
    } else if (argument === "--perspective") {
      const value = args[index + 1];
      if (!value) throw new Error("--perspective requires a value");
      perspective = parseWorkPerspective(value);
      index += 1;
    } else if (argument === "--help" || argument === "-h") {
      return { help: true, json: false };
    } else {
      throw new Error(`unknown argument: ${argument}`);
    }
  }

  return perspective
    ? { help: false, json, perspective }
    : { help: false, json };
}

export function helpText(): string {
  return [
    "Tramuri TypeScript · seleção de perspectiva",
    "",
    "Usage:",
    "  tramuri-cli perspective choose [--perspective <value>] [--json]",
    "",
    "Values:",
    "  product_engineer  Produto, problema e resultado para usuários.",
    "  fde               Necessidade do cliente, sistemas, restrições e aceite.",
    "  mixed             Produto ligado a uma necessidade de cliente.",
    "",
    "Without --perspective, the command requires an interactive terminal.",
    "This first TypeScript slice records the choice in its output; it does not create a Mission.",
  ].join("\n");
}

export async function runCli(args: string[]): Promise<number> {
  try {
    const command = parseCommand(args);
    if (command.help) {
      process.stdout.write(`${helpText()}\n`);
      return 0;
    }

    const selectionMethod: SelectionMethod = command.perspective
      ? "explicit_argument"
      : "interactive_terminal";
    const perspective = command.perspective ?? await promptWorkPerspective();
    const result = makeSelection(perspective, selectionMethod);

    if (command.json) {
      process.stdout.write(`${JSON.stringify(result, null, 2)}\n`);
    } else {
      const name = perspective === "product_engineer"
        ? "Product Engineer"
        : perspective === "fde"
          ? "FDE"
          : "Ambos";
      process.stdout.write(`Perspectiva escolhida: ${name}.\n`);
      process.stdout.write("Nenhuma Mission foi criada nesta etapa inicial do CLI TypeScript.\n");
    }
    return 0;
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    process.stderr.write(`tramuri-cli: ${message}\n`);
    return 2;
  }
}

const invokedPath = process.argv[1];
if (invokedPath && resolve(invokedPath) === resolve(fileURLToPath(import.meta.url))) {
  process.exitCode = await runCli(process.argv.slice(2));
}
