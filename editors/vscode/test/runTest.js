const path = require("node:path");
const { runTests } = require("@vscode/test-electron");

async function main() {
  delete process.env.ELECTRON_RUN_AS_NODE;
  const extensionDevelopmentPath = path.resolve(__dirname, "..");
  const extensionTestsPath = path.resolve(__dirname, "suite", "index");

  await runTests({
    extensionDevelopmentPath,
    extensionTestsPath,
    launchArgs: ["--disable-extensions", "--verbose"],
    version: "1.85.2",
  });
}

main().catch((error) => {
  console.error(error);
  process.exit(1);
});
