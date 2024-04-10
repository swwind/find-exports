export const find_exports: (
  source: string,
  callees: string[]
) => {
  callee: string;
  name: string;
}[];
