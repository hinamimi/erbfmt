const assert = require("node:assert");
const { parseCommandSetting, splitCommandSetting } = require("../out/command");

assert.deepStrictEqual(parseCommandSetting("bundle exec erbfmt"), {
  command: "bundle",
  arguments: ["exec", "erbfmt"],
});

assert.deepStrictEqual(parseCommandSetting("'bundle' \"exec\" erbfmt"), {
  command: "bundle",
  arguments: ["exec", "erbfmt"],
});

assert.deepStrictEqual(parseCommandSetting('"/path with spaces/erbfmt"'), {
  command: "/path with spaces/erbfmt",
  arguments: [],
});

assert.deepStrictEqual(splitCommandSetting("erbfmt --some-flag"), ["erbfmt", "--some-flag"]);
assert.deepStrictEqual(splitCommandSetting(""), []);

console.log("VSCode command parsing test passed.");
