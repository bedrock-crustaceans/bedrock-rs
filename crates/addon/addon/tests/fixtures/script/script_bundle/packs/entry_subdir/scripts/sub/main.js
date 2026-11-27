import { world } from "@minecraft/server";
for (const [spec, load] of [
  ["./x.js", () => import("./x.js")],
  ["x.js", () => import("x.js")],
  ["../top.js", () => import("../top.js")],
  ["sub/x.js", () => import("sub/x.js")],
]) load().then((m) => console.warn("[entry_subdir] OK " + spec + " -> " + m.v));
