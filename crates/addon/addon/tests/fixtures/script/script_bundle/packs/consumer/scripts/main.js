import("libx.js").catch((e) => console.warn("[consumer] FAIL " + e));
import("./libx.js").catch((e) => console.warn("[consumer] FAIL " + e));
import("../../libpack/scripts/libx.js").catch((e) => console.warn("[consumer] FAIL " + e));
