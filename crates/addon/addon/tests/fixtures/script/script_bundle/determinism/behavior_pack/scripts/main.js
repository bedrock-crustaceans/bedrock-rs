import { v } from "./lib/b.js";
import("./lib/c").then((m) => console.warn(v + m.v));
