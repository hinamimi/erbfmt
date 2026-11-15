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
  test("quick fix formats only the selected change", async () => {
    const uri = await writeTempHtmlErb(
      "<div>\n<p>First</p>\n</div>\n<hr>\n<div>\n<p>Second</p>\n</div>\n",
    );
    const document = await vscode.workspace.openTextDocument(uri);
    await vscode.window.showTextDocument(document);
    await vscode.commands.executeCommand("erbfmt.lintDocument");
    const diagnostic = vscode.languages.getDiagnostics(uri).find((item) => item.code === "format");
    assert(diagnostic);
    const actions = await quickFixes(uri, diagnostic.range);
    const fix = actions.find((action) => action.title === "erbfmt: Format this change");
    assert(fix);
    assert(await applyQuickFix(fix));
    assert.equal(
      document.getText(),
      "<div>\n  <p>First</p>\n</div>\n<hr>\n<div>\n<p>Second</p>\n</div>\n",
    );
  });

  test("quick fix uses Rust assist edits with Unicode byte offsets", async () => {
    const original = "<p>日本語😀</p>\n<input title='first'>\n<input title='second'>\n";
    const uri = await writeTempHtmlErb(original);
    const document = await vscode.workspace.openTextDocument(uri);
    await vscode.window.showTextDocument(document);
    await vscode.commands.executeCommand("erbfmt.lintDocument");
    const diagnostic = vscode.languages
      .getDiagnostics(uri)
      .find((item) => item.code === "noNonDoubleQuotedHtmlAttributeValue");
    assert(diagnostic);
    const actions = await quickFixes(uri, diagnostic.range);
    const fix = actions.find(
      (action) => action.title.includes("double quote") || action.title.includes("double-quote"),
    );
    assert(fix, JSON.stringify(actions.map((action) => action.title)));
    assert(await applyQuickFix(fix));
    assert.equal(document.getText(), original.replace("'first'", '"first"'));
  });

  test("quick fix ignores only the selected lint rule and line", async () => {
    const uri = await writeTempHtmlErb("<center>First</center>\n<center>Second</center>\n");
    const document = await vscode.workspace.openTextDocument(uri);
    await vscode.window.showTextDocument(document);
    await vscode.commands.executeCommand("erbfmt.lintDocument");
    const diagnostic = vscode.languages
      .getDiagnostics(uri)
      .find((item) => item.code === "noDeprecatedHtmlTag");
    assert(diagnostic);
    const actions = await quickFixes(uri, diagnostic.range);
    const ignore = actions.find(
      (action) => action.title === "erbfmt: Ignore noDeprecatedHtmlTag on this line",
    );
    assert(ignore);
    assert(await applyQuickFix(ignore));
    assert(document.getText().startsWith("<%# erbfmt-ignore lint/noDeprecatedHtmlTag:"));
    await vscode.commands.executeCommand("erbfmt.lintDocument");
    const remaining = vscode.languages
      .getDiagnostics(uri)
      .filter((item) => item.code === "noDeprecatedHtmlTag");
    assert(remaining.length > 0);
    assert(remaining.every((item) => item.range.start.line === 2));
  });

  test("quick fix ignores formatting for the next node", async () => {
    const uri = await writeTempHtmlErb("<div>\n<p>Keep</p>\n</div>\n");
    const document = await vscode.workspace.openTextDocument(uri);
    await vscode.window.showTextDocument(document);
    await vscode.commands.executeCommand("erbfmt.lintDocument");
    const diagnostic = vscode.languages.getDiagnostics(uri).find((item) => item.code === "format");
    assert(diagnostic);
    const actions = await quickFixes(uri, diagnostic.range);
    const ignore = actions.find(
      (action) => action.title === "erbfmt: Ignore formatting for the next node",
    );
    assert(ignore);
    assert(await applyQuickFix(ignore));
    await vscode.commands.executeCommand("erbfmt.formatDocument");
    assert(document.getText().includes("\n<p>Keep</p>\n"));
    await vscode.commands.executeCommand("erbfmt.lintDocument");
    assert(!vscode.languages.getDiagnostics(uri).some((item) => item.code === "format"));
  });

  test("does not offer ignore for parser errors or inside multiline attributes", async () => {
    for (const source of ["<div>\n", "<input\n  title='first'\n>\n"]) {
      const uri = await writeTempHtmlErb(source);
      const document = await vscode.workspace.openTextDocument(uri);
      await vscode.window.showTextDocument(document);
      await vscode.commands.executeCommand("erbfmt.lintDocument");
      const actions = await quickFixes(uri, new vscode.Range(0, 0, document.lineCount, 0));
      assert(
        !actions.some((action) =>
          action.title.startsWith(
            source.startsWith("<input")
              ? "erbfmt: Ignore noNonDoubleQuotedHtmlAttributeValue"
              : "erbfmt: Ignore",
          ),
        ),
      );
    }
  });

  test("offers independent fix and ignore actions for empty ERB tags", async () => {
    const uri = await writeTempHtmlErb("<% %>\n<p>Hello</p>\n");
    const document = await vscode.workspace.openTextDocument(uri);
    await vscode.window.showTextDocument(document);
    await vscode.commands.executeCommand("erbfmt.lintDocument");
    const diagnostic = vscode.languages
      .getDiagnostics(uri)
      .find((item) => item.code === "emptyErbCodeTag");
    assert(diagnostic);
    const actions = await quickFixes(uri, diagnostic.range);
    assert(actions.some((action) => action.title === "erbfmt: Remove empty ERB tag"));
    const ignore = actions.find(
      (action) => action.title === "erbfmt: Ignore emptyErbCodeTag on this line",
    );
    assert(ignore);
    assert(await applyQuickFix(ignore));
    assert(!vscode.languages.getDiagnostics(uri).some((item) => item.code === "emptyErbCodeTag"));
  });

  test("stale menu actions cannot edit a changed document", async () => {
    const uri = await writeTempHtmlErb("<div>\n<p>Hello</p>\n</div>\n");
    const document = await vscode.workspace.openTextDocument(uri);
    const editor = await vscode.window.showTextDocument(document);
    await vscode.commands.executeCommand("erbfmt.lintDocument");
    const diagnostic = vscode.languages.getDiagnostics(uri).find((item) => item.code === "format");
    const fix = (await quickFixes(uri, diagnostic.range)).find(
      (action) => action.title === "erbfmt: Format this change",
    );
    assert(fix);
    await editor.edit((edit) => edit.insert(new vscode.Position(0, 0), "<hr>\n"));
    const changed = document.getText();
    assert.equal(await applyQuickFix(fix), false);
    assert.equal(document.getText(), changed);
    await waitUntil(() =>
      vscode.languages.getDiagnostics(uri).some((item) => item.code === "format"),
    );
    const fresh = vscode.languages.getDiagnostics(uri).find((item) => item.code === "format");
    assert(
      (await quickFixes(uri, fresh.range)).some(
        (action) => action.title === "erbfmt: Format this change",
      ),
    );
  });

  test("deduplicates the same rule ignore on one line", async () => {
    const uri = await writeTempHtmlErb("<center><font>Legacy</font></center>\n");
    const document = await vscode.workspace.openTextDocument(uri);
    await vscode.window.showTextDocument(document);
    await vscode.commands.executeCommand("erbfmt.lintDocument");
    const actions = await quickFixes(uri, new vscode.Range(0, 0, 0, 45));
    assert.equal(
      actions.filter((action) => action.title === "erbfmt: Ignore noDeprecatedHtmlTag on this line")
        .length,
      1,
    );
  });

  test("adding a rule ignore preserves an existing rule ignore", async () => {
    const original =
      "<%# erbfmt-ignore lint/noDeprecatedHtmlTag: legacy %>\n<center title='value'>Legacy</center>\n";
    const uri = await writeTempHtmlErb(original);
    const document = await vscode.workspace.openTextDocument(uri);
    await vscode.window.showTextDocument(document);
    await vscode.commands.executeCommand("erbfmt.lintDocument");
    const diagnostic = vscode.languages
      .getDiagnostics(uri)
      .find((item) => item.code === "noNonDoubleQuotedHtmlAttributeValue");
    assert(diagnostic);
    const action = (await quickFixes(uri, diagnostic.range)).find(
      (item) => item.title === "erbfmt: Ignore noNonDoubleQuotedHtmlAttributeValue on this line",
    );
    assert(action);
    assert(await applyQuickFix(action));
    const diagnostics = vscode.languages.getDiagnostics(uri);
    assert(
      !diagnostics.some(
        (item) =>
          item.code === "noDeprecatedHtmlTag" ||
          item.code === "noNonDoubleQuotedHtmlAttributeValue",
      ),
    );
    assert(document.getText().includes("legacy %>"));
  });

  test("does not insert ignore comments inside raw text or multiline Ruby strings", async () => {
    for (const original of [
      '<script>\nconst html = "<center>Legacy</center>";\n</script>\n',
      '<% value = "\n<center>Legacy</center>\n" %>\n',
    ]) {
      const uri = await writeTempHtmlErb(original);
      const document = await vscode.workspace.openTextDocument(uri);
      await vscode.window.showTextDocument(document);
      await vscode.commands.executeCommand("erbfmt.lintDocument");
      const actions = await quickFixes(uri, new vscode.Range(1, 0, 1, 30));
      assert(!actions.some((action) => action.title.startsWith("erbfmt: Ignore")));
    }
  });

  test("individual formatting preserves CRLF and other changes", async () => {
    const original =
      "<div>\r\n<p>日本語😀</p>\r\n</div>\r\n<hr>\r\n<div>\r\n<p>Second</p>\r\n</div>\r\n";
    const uri = await writeTempHtmlErb(original);
    const tempDir = path.dirname(uri.fsPath);
    const config = path.join(tempDir, "erbfmt.json");
    await fs.writeFile(config, JSON.stringify({ formatter: { lineEnding: "crlf" } }));
    const settings = vscode.workspace.getConfiguration("erbfmt");
    const previous = settings.get("configPath");
    await settings.update("configPath", config, vscode.ConfigurationTarget.Global);
    try {
      const document = await vscode.workspace.openTextDocument(uri);
      await vscode.window.showTextDocument(document);
      await vscode.commands.executeCommand("erbfmt.lintDocument");
      const diagnostic = vscode.languages
        .getDiagnostics(uri)
        .find((item) => item.code === "format");
      const fix = (await quickFixes(uri, diagnostic.range)).find(
        (action) => action.title === "erbfmt: Format this change",
      );
      assert(fix);
      assert(await applyQuickFix(fix));
      assert.equal(document.getText(), original.replace("<p>日本語😀", "  <p>日本語😀"));
    } finally {
      await settings.update("configPath", previous, vscode.ConfigurationTarget.Global);
    }
  });

  test("does not offer fixes for stale diagnostics", async () => {
    const uri = await writeTempHtmlErb("<div>\n<p>Hello</p>\n</div>\n");
    const document = await vscode.workspace.openTextDocument(uri);
    const editor = await vscode.window.showTextDocument(document);
    await vscode.commands.executeCommand("erbfmt.lintDocument");
    await editor.edit((edit) => edit.insert(new vscode.Position(0, 0), "\n"));
    const actions = await quickFixes(uri, new vscode.Range(0, 0, 4, 0));
    assert.deepEqual(
      actions.filter((action) => action.title.startsWith("erbfmt:")),
      [],
    );
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

async function quickFixes(uri, range) {
  return vscode.commands.executeCommand("vscode.executeCodeActionProvider", uri, range, "quickfix");
}

async function applyQuickFix(action) {
  return vscode.commands.executeCommand(action.command.command, ...action.command.arguments);
}

async function waitUntil(predicate) {
  for (let attempt = 0; attempt < 40; attempt += 1) {
    if (predicate()) return;
    await delay(100);
  }
  assert(predicate(), "timed out waiting for diagnostics");
}
