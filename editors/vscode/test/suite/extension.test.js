const assert = require("node:assert");
const fs = require("node:fs/promises");
const os = require("node:os");
const path = require("node:path");
const vscode = require("vscode");

const extensionRoot = path.resolve(__dirname, "..", "..");
const repoRoot = path.resolve(extensionRoot, "..", "..");
const extensionId = "erbfmt.erbfmt-vscode";
const tempDirs = [];

suite("erbfmt VSCode extension", () => {
  suiteSetup(async () => {
    const extension = vscode.extensions.getExtension(extensionId);
    assert(extension, `${extensionId} should be available in the extension host`);
    await extension.activate();
    await configureErbfmtCommand();
  });

  suiteTeardown(async () => {
    await Promise.all(tempDirs.map((tempDir) => fs.rm(tempDir, { recursive: true, force: true })));
  });

  test("formats html-erb documents through the document formatter provider", async () => {
    const uri = await writeTempHtmlErb("<% if user %>\n<p>Hello</p>\n<% end %>\n");
    const document = await vscode.workspace.openTextDocument(uri);

    assert.strictEqual(document.languageId, "html-erb");

    const edits = await vscode.commands.executeCommand(
      "vscode.executeFormatDocumentProvider",
      uri,
      { insertSpaces: true, tabSize: 2 },
    );

    assert(Array.isArray(edits), "format provider should return edits");
    assert(edits.length > 0, "format provider should return a replacement edit");

    const workspaceEdit = new vscode.WorkspaceEdit();
    for (const edit of edits) {
      workspaceEdit.replace(uri, edit.range, edit.newText);
    }

    assert(await vscode.workspace.applyEdit(workspaceEdit));
    assert.strictEqual(document.getText(), "<% if user %>\n  <p>Hello</p>\n<% end %>\n");
  });

  for (const excludedPath of ["tmp/input.html.erb", "app/views/generated/input.html.erb"]) {
    test(`skips formatting and diagnostics for excluded ${excludedPath}`, async () => {
      const tempDir = await fs.mkdtemp(path.join(os.tmpdir(), "erbfmt-vscode-ignore-"));
      tempDirs.push(tempDir);
      const configPath = path.join(tempDir, "erbfmt.json");
      await fs.writeFile(
        configPath,
        JSON.stringify({
          files: { includes: ["app/views/**/*.html.erb", "!app/views/generated/**"] },
        }),
      );
      const settings = vscode.workspace.getConfiguration("erbfmt");
      const previousConfig = settings.get("configPath");
      await settings.update("configPath", configPath, vscode.ConfigurationTarget.Global);
      try {
        const filePath = path.join(tempDir, excludedPath);
        await fs.mkdir(path.dirname(filePath), { recursive: true });
        const original = "<div>\n<p>Hello</p>\n</div>\n";
        await fs.writeFile(filePath, original);
        const uri = vscode.Uri.file(filePath);
        const document = await vscode.workspace.openTextDocument(uri);
        const editor = await vscode.window.showTextDocument(document);
        const edits = await vscode.commands.executeCommand(
          "vscode.executeFormatDocumentProvider",
          uri,
          { insertSpaces: true, tabSize: 2 },
        );
        assert(!edits || edits.length === 0, "excluded documents should have no formatting edits");
        await vscode.commands.executeCommand("erbfmt.formatDocument");
        assert.strictEqual(document.getText(), original);

        const editorSettings = vscode.workspace.getConfiguration("editor", uri);
        const previousFormatOnSave = editorSettings.inspect("formatOnSave").globalValue;
        await editorSettings.update("formatOnSave", true, vscode.ConfigurationTarget.Global);
        try {
          await editor.edit((edit) => edit.insert(new vscode.Position(2, 0), "<p>Unsaved</p>\n"));
          const unsaved = document.getText();
          assert(await document.save());
          assert.strictEqual(document.getText(), unsaved);
          assert.strictEqual(await fs.readFile(filePath, "utf8"), unsaved);
        } finally {
          await editorSettings.update(
            "formatOnSave",
            previousFormatOnSave,
            vscode.ConfigurationTarget.Global,
          );
        }

        // Invalid syntax in an excluded buffer must not produce lint or format diagnostics.
        await editor.edit((edit) => edit.insert(new vscode.Position(0, 0), "<% while user %>\n"));
        await vscode.commands.executeCommand("erbfmt.lintDocument");
        assert.deepStrictEqual(vscode.languages.getDiagnostics(uri), []);
      } finally {
        await settings.update("configPath", previousConfig, vscode.ConfigurationTarget.Global);
      }
    });
  }

  test("publishes lint diagnostics on the offending ERB tag range", async () => {
    const uri = await writeTempHtmlErb(
      "<p>Before</p>\n  <% while job.running? %>\n<p>Waiting</p>\n",
    );
    const document = await vscode.workspace.openTextDocument(uri);
    await vscode.window.showTextDocument(document);

    await vscode.commands.executeCommand("erbfmt.lintDocument");
    const diagnostics = (await waitForDiagnostics(uri)).filter((diagnostic) =>
      diagnostic.message.includes("unsupported ERB block starter"),
    );

    assert.strictEqual(diagnostics.length, 1);
    assert.match(diagnostics[0].message, /unsupported ERB block starter/);
    assert.strictEqual(diagnostics[0].range.start.line, 1);
    assert.strictEqual(diagnostics[0].range.start.character, 2);
    assert.strictEqual(diagnostics[0].range.end.line, 1);
    assert.strictEqual(diagnostics[0].range.end.character, 3);
  });
});

async function configureErbfmtCommand() {
  const binaryName = process.platform === "win32" ? "erbfmt.exe" : "erbfmt";
  const binaryPath = path.join(repoRoot, "target", "debug", binaryName);

  await fs.access(binaryPath);

  const config = vscode.workspace.getConfiguration("erbfmt");
  await config.update("command", binaryPath, vscode.ConfigurationTarget.Global);
  await config.update("arguments", [], vscode.ConfigurationTarget.Global);
  await config.update(
    "configPath",
    path.join(repoRoot, "erbfmt.json"),
    vscode.ConfigurationTarget.Global,
  );
  await config.update("lint.enabled", true, vscode.ConfigurationTarget.Global);
}

async function writeTempHtmlErb(content) {
  const tempDir = await fs.mkdtemp(path.join(os.tmpdir(), "erbfmt-vscode-test-"));
  tempDirs.push(tempDir);

  const filePath = path.join(tempDir, "input.html.erb");
  await fs.writeFile(filePath, content, "utf8");

  return vscode.Uri.file(filePath);
}

async function waitForDiagnostics(uri) {
  for (let attempt = 0; attempt < 20; attempt += 1) {
    const diagnostics = vscode.languages.getDiagnostics(uri);
    if (diagnostics.length > 0) {
      return diagnostics;
    }

    await delay(100);
  }

  return vscode.languages.getDiagnostics(uri);
}

function delay(ms) {
  return new Promise((resolve) => {
    setTimeout(resolve, ms);
  });
}
