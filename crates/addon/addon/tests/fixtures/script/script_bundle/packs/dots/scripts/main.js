for (const load of [() => import("../escape"), () => import("./lib/."), () => import("...")]) load().catch((e) => console.warn("[rv] FAIL " + e));
