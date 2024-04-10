# `@swwind/find-exports`

**This module is ESM only and uses WASM, supports NodeJS and browsers out of box.**

Parse ESM code and find exports which matches the following pattern.

```ts
export const xxx = loader$(/* ... */);
export const xxx = action$(/* ... */);
```

where callee names (`loader$` and `action$`, etc.) can be specified.

## Features

- [x] TypeScript
- [x] JSX / TSX

## Example

```ts
import { find_exports } from "@swwind/find-exports";

const code = `
export const one = loader$(() => { return true; });
export let two = action$(() => { return false; });
export var the = loader$(() => { return false; });

// below may works, but will not match
const none = loader$(() => {}); export { none };
export const none = (0, loader$)(() => {});
export const [none = loader$(() => {})] = [];
`;

const found = find_exports(code, ["loader$", "action$"]);
// => found = [
// =>   { callee: "loader$", name: "one" },
// =>   { callee: "action$", name: "two" },
// =>   { callee: "loader$", name: "the" },
// => ]
```
