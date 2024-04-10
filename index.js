import init, { find_exports as _find_exports } from "./pkg/find_exports.js";

await init();

export const find_exports = (source, callees) => {
  return _find_exports(source, callees).map((x) => {
    let [callee, name] = x.split("/", 2);
    return { callee, name };
  });
};
