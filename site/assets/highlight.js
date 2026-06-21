"use strict";

const script = document.createElement("script");
script.src =
  "https://cdn.jsdelivr.net/gh/highlightjs/cdn-release@11.11.1/build/highlight.min.js";
script.integrity =
  "sha384-RH2xi4eIQ/gjtbs9fUXM68sLSi99C7ZWBRX1vDrVv6GQXRibxXLbwO2NGZB74MbU";
script.crossOrigin = "anonymous";
script.addEventListener("load", () => window.hljs?.highlightAll());
document.head.append(script);
