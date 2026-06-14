const childProcess = require("child_process");
const fs = require("fs/promises");
const os = require("os");
const path = require("path");
const vscode = require("vscode");

const FORMATTER_SELECTOR = [
  { scheme: "file", language: "erb" },
  { scheme: "file", language: "html-erb" },
];

function activate(context) {
  const diagnostics = vscode.languages.createDiagnosticCollection("erbfmt");
  const provider = {
    async provideDocumentFormattingEdits(document, _options, token) {
      const formatted = await formatDocument(document, token);
      const fullRange = new vscode.Range(
        document.positionAt(0),
        document.positionAt(document.getText().length),
      );

      return [vscode.TextEdit.replace(fullRange, formatted)];
    },
  };

  context.subscriptions.push(
    diagnostics,
    vscode.languages.registerDocumentFormattingEditProvider(
      FORMATTER_SELECTOR,
      provider,
    ),
    vscode.commands.registerCommand("erbfmt.formatDocument", async () => {
      if (!vscode.window.activeTextEditor) {
        return;
      }

      await vscode.commands.executeCommand("editor.action.formatDocument");
    }),
    vscode.commands.registerCommand("erbfmt.lintDocument", async () => {
      const editor = vscode.window.activeTextEditor;
      if (!editor) {
        return;
      }

      await lintDocument(editor.document, diagnostics, createNullToken());
    }),
    vscode.workspace.onDidOpenTextDocument((document) => {
      void lintDocument(document, diagnostics, createNullToken());
    }),
    vscode.workspace.onDidSaveTextDocument((document) => {
      void lintDocument(document, diagnostics, createNullToken());
    }),
    vscode.workspace.onDidCloseTextDocument((document) => {
      diagnostics.delete(document.uri);
    }),
  );

  for (const document of vscode.workspace.textDocuments) {
    void lintDocument(document, diagnostics, createNullToken());
  }
}

function deactivate() {}

async function formatDocument(document, token) {
  if (document.uri.scheme !== "file") {
    throw new Error("erbfmt can only format files on disk.");
  }

  if (token.isCancellationRequested) {
    return document.getText();
  }

  const context = getCommandContext(document);

  const tempDir = await fs.mkdtemp(path.join(os.tmpdir(), "erbfmt-vscode-"));
  const tempFile = path.join(tempDir, path.basename(document.uri.fsPath));

  try {
    await fs.writeFile(tempFile, document.getText(), "utf8");

    const args = await buildErbfmtArgs(context, document, [tempFile]);

    const result = await execFile(context.command, args, { cwd: context.cwd }, token);
    if (result.exitCode !== 0) {
      const detail = result.stderr.trim() || `exit code ${result.exitCode}`;
      throw new Error(`erbfmt failed: ${detail}`);
    }

    const { stdout } = result;
    return stdout;
  } finally {
    await fs.rm(tempDir, { recursive: true, force: true });
  }
}

async function lintDocument(document, diagnostics, token) {
  if (!isSupportedDocument(document)) {
    diagnostics.delete(document.uri);
    return;
  }

  const settings = vscode.workspace.getConfiguration("erbfmt", document.uri);
  if (!settings.get("lint.enabled", true)) {
    diagnostics.delete(document.uri);
    return;
  }

  const context = getCommandContext(document);
  const tempDir = await fs.mkdtemp(path.join(os.tmpdir(), "erbfmt-vscode-"));
  const tempFile = path.join(tempDir, path.basename(document.uri.fsPath));

  try {
    await fs.writeFile(tempFile, document.getText(), "utf8");
    const args = await buildErbfmtArgs(context, document, ["--lint", tempFile]);
    const result = await execFile(context.command, args, { cwd: context.cwd }, token);

    if (result.exitCode === 0) {
      diagnostics.delete(document.uri);
      return;
    }

    diagnostics.set(
      document.uri,
      parseDiagnostics(document, tempFile, result.stderr),
    );
  } catch (error) {
    diagnostics.set(document.uri, [
      new vscode.Diagnostic(
        firstCharacterRange(document),
        error.message,
        vscode.DiagnosticSeverity.Error,
      ),
    ]);
  } finally {
    await fs.rm(tempDir, { recursive: true, force: true });
  }
}

function getCommandContext(document) {
  const workspaceFolder = vscode.workspace.getWorkspaceFolder(document.uri);
  const cwd = workspaceFolder?.uri.fsPath ?? path.dirname(document.uri.fsPath);
  const settings = vscode.workspace.getConfiguration("erbfmt", document.uri);
  const command = settings.get("command", "erbfmt");
  const configuredArguments = settings.get("arguments", []);
  const extraArguments = Array.isArray(configuredArguments)
    ? configuredArguments.filter((argument) => typeof argument === "string")
    : [];

  return {
    command,
    cwd,
    extraArguments,
    settings,
    workspaceFolder,
  };
}

async function buildErbfmtArgs(context, document, trailingArguments) {
  const args = [...context.extraArguments];
  const configPath = await resolveConfigPath(
    context.settings,
    document,
    context.workspaceFolder,
  );

  if (configPath) {
    args.push("--config", configPath);
  }

  args.push(...trailingArguments);
  return args;
}

async function resolveConfigPath(settings, document, workspaceFolder) {
  const configuredPath = settings.get("configPath", "").trim();
  if (configuredPath) {
    if (path.isAbsolute(configuredPath)) {
      return configuredPath;
    }

    const base = workspaceFolder?.uri.fsPath ?? path.dirname(document.uri.fsPath);
    return path.resolve(base, configuredPath);
  }

  return findNearestConfig(path.dirname(document.uri.fsPath));
}

async function findNearestConfig(startDirectory) {
  let current = startDirectory;

  while (true) {
    const candidate = path.join(current, "erbfmt.json");
    if (await isFile(candidate)) {
      return candidate;
    }

    const parent = path.dirname(current);
    if (parent === current) {
      return undefined;
    }

    current = parent;
  }
}

async function isFile(filePath) {
  try {
    const stat = await fs.stat(filePath);
    return stat.isFile();
  } catch (_error) {
    return false;
  }
}

function parseDiagnostics(document, filePath, stderr) {
  const diagnostics = [];

  for (const rawLine of stderr.split(/\r?\n/)) {
    const line = rawLine.trim();
    if (!line) {
      continue;
    }

    const message = line.startsWith(`${filePath}: `)
      ? line.slice(filePath.length + 2)
      : line;
    const diagnostic = new vscode.Diagnostic(
      rangeFromMessage(document, message),
      message,
      vscode.DiagnosticSeverity.Error,
    );
    diagnostic.source = "erbfmt";
    diagnostics.push(diagnostic);
  }

  return diagnostics;
}

function rangeFromMessage(document, message) {
  const match = message.match(/ at line (\d+), column (\d+)$/);
  if (!match) {
    return firstCharacterRange(document);
  }

  const line = Math.max(0, Math.min(Number(match[1]) - 1, document.lineCount - 1));
  const lineText = document.lineAt(line).text;
  const column = Math.max(0, Math.min(Number(match[2]) - 1, lineText.length));
  const endColumn = Math.min(column + 1, lineText.length);

  return new vscode.Range(line, column, line, endColumn);
}

function firstCharacterRange(document) {
  if (document.lineCount === 0) {
    return new vscode.Range(0, 0, 0, 0);
  }

  const firstLineLength = document.lineAt(0).text.length;
  return new vscode.Range(0, 0, 0, Math.min(1, firstLineLength));
}

function isSupportedDocument(document) {
  return (
    document.uri.scheme === "file" &&
    FORMATTER_SELECTOR.some((selector) => selector.language === document.languageId)
  );
}

function createNullToken() {
  return {
    isCancellationRequested: false,
    onCancellationRequested: () => ({ dispose() {} }),
  };
}

function execFile(command, args, options, token) {
  return new Promise((resolve) => {
    const child = childProcess.execFile(
      command,
      args,
      {
        ...options,
        maxBuffer: 16 * 1024 * 1024,
      },
      (error, stdout, stderr) => {
        resolve({
          stdout,
          stderr,
          exitCode: error?.code ?? 0,
        });
      },
    );

    token.onCancellationRequested(() => {
      child.kill();
      resolve({
        stdout: "",
        stderr: "erbfmt formatting was cancelled.",
        exitCode: 1,
      });
    });
  });
}

module.exports = {
  activate,
  deactivate,
};
