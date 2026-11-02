try { console.warn("[eval_nocap] eval=" + eval("1+1")); } catch (e) { console.warn("[eval_nocap] eval threw " + e); }
try { console.warn("[eval_nocap] Function=" + new Function("return 3")()); } catch (e) { console.warn("[eval_nocap] Function threw " + e); }
