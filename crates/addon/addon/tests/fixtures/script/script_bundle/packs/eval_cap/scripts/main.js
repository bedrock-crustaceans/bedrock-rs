try { console.warn("[eval_cap] eval=" + eval("1+1")); } catch (e) { console.warn("[eval_cap] eval threw " + e); }
try { console.warn("[eval_cap] Function=" + new Function("return 3")()); } catch (e) { console.warn("[eval_cap] Function threw " + e); }
