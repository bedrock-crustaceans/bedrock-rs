import("shared_lib.js").catch((e) => console.warn("[consumer2] FAIL " + e));
import("./shared_lib.js").catch((e) => console.warn("[consumer2] FAIL " + e));
