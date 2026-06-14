const Mocha = require("mocha");
const path = require("node:path");

exports.run = () =>
  new Promise((resolve, reject) => {
    const mocha = new Mocha({
      color: true,
      timeout: 30_000,
      ui: "tdd",
    });

    mocha.addFile(path.resolve(__dirname, "extension.test.js"));

    mocha.run((failures) => {
      if (failures > 0) {
        reject(new Error(`${failures} VSCode extension-host test(s) failed.`));
      } else {
        resolve();
      }
    });
  });
