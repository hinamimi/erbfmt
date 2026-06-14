const assert = require("node:assert");
const fs = require("node:fs");
const path = require("node:path");

const extensionDir = path.resolve(__dirname, "..");
const packageJson = JSON.parse(fs.readFileSync(path.join(extensionDir, "package.json"), "utf8"));
const extensionSource = fs.readFileSync(path.join(extensionDir, "src", "extension.ts"), "utf8");

assert.strictEqual(packageJson.main, "./out/extension.js");
assert.strictEqual(packageJson.scripts.compile, "tsc -p .");
assert(packageJson.activationEvents.includes("onLanguage:erb"));
assert(packageJson.activationEvents.includes("onLanguage:html-erb"));
assert(packageJson.contributes.languages.some((language) => language.id === "html-erb"));
assert(
  packageJson.contributes.commands.some((command) => command.command === "erbfmt.formatDocument"),
);
assert(
  packageJson.contributes.commands.some((command) => command.command === "erbfmt.lintDocument"),
);
assert(packageJson.contributes.configuration.properties["erbfmt.command"]);
assert(packageJson.contributes.configuration.properties["erbfmt.arguments"]);
assert(packageJson.contributes.configuration.properties["erbfmt.configPath"]);
assert(packageJson.contributes.configuration.properties["erbfmt.lint.enabled"]);
assert.strictEqual(
  packageJson.contributes.configurationDefaults["[html-erb]"]["editor.defaultFormatter"],
  "erbfmt.erbfmt-vscode",
);
assert(extensionSource.includes("registerDocumentFormattingEditProvider"));
assert(extensionSource.includes("createDiagnosticCollection"));
assert(extensionSource.includes("--lint"));
assert(extensionSource.includes("fullDocumentRange"));
assert(extensionSource.includes("childProcess.execFile"));

console.log("VSCode extension smoke test passed.");
