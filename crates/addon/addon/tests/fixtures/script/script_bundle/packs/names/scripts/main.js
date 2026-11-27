import { world } from "@minecraft/server";
const specs = {
  "./my file.js": () => import("./my file.js"),
  "./ünï.js": () => import("./ünï.js"),
  "./lib2/index.js": () => import("./lib2/index.js"),
  "./.hidden.js": () => import("./.hidden.js"),
  "./a.b.js": () => import("./a.b.js"),
  "./a.b": () => import("./a.b"),
  "./node_modules/pkg/index.js": () => import("./node_modules/pkg/index.js"),
  "./lib2": () => import("./lib2"),
  "pkg": () => import("pkg"),
};
for (const spec in specs) specs[spec]().then((m) => console.warn("[names] OK " + spec + " -> " + m.v), (e) => console.warn("[names] FAIL " + spec + " -> " + e));
