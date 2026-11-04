export const WORK_PERSPECTIVES = [
  {
    value: "product_engineer",
    label: "Product Engineer",
    description: "Produto, problema e resultado para usuários.",
  },
  {
    value: "fde",
    label: "FDE",
    description: "Necessidade do cliente, sistemas, restrições e aceite.",
  },
  {
    value: "mixed",
    label: "Ambos",
    description: "Evolução do produto ligada a uma necessidade de cliente.",
  },
] as const;

export type WorkPerspective = (typeof WORK_PERSPECTIVES)[number]["value"];
export type SelectionMethod = "explicit_argument" | "interactive_terminal";

export interface WorkPerspectiveSelection {
  schema: "tramuri.work-perspective-selection.v1";
  work_perspective: WorkPerspective;
  selection_method: SelectionMethod;
  mission_created: false;
}

export function parseWorkPerspective(value: string): WorkPerspective {
  const choice = WORK_PERSPECTIVES.find((item) => item.value === value);
  if (!choice) {
    throw new Error(
      "perspective must be product_engineer, fde, or mixed",
    );
  }
  return choice.value;
}

export function moveSelection(current: number, offset: number): number {
  const count = WORK_PERSPECTIVES.length;
  return (current + offset + count) % count;
}

export function makeSelection(
  perspective: WorkPerspective,
  method: SelectionMethod,
): WorkPerspectiveSelection {
  return {
    schema: "tramuri.work-perspective-selection.v1",
    work_perspective: perspective,
    selection_method: method,
    mission_created: false,
  };
}
