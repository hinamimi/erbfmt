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
  );
}

function deactivate() {}

async function formatDocument(document, token) {
  if (document.uri.scheme !== "file") {
    throw new Error("erbfmt can only format files on disk.");
  }

  if (token.isCancellationRequested) {
    return document.getText();
  }

  const workspaceFolder = vscode.workspace.getWorkspaceFolder(document.uri);
  const cwd = workspaceFolder?.uri.fsPath ?? path.dirname(document.uri.fsPath);
  const settings = vscode.workspace.getConfiguration("erbfmt", document.uri);
  const command = settings.get("command", "erbfmt");
  const configuredArguments = settings.get("arguments", []);
  const extraArguments = Array.isArray(configuredArguments)
    ? configuredArguments.filter((argument) => typeof argument === "string")
    : [];

  const tempDir = await fs.mkdtemp(path.join(os.tmpdir(), "erbfmt-vscode-"));
  const tempFile = path.join(tempDir, path.basename(document.uri.fsPath));

  try {
    await fs.writeFile(tempFile, document.getText(), "utf8");

    const args = [...extraArguments];
    const configPath = await resolveConfigPath(settings, document, workspaceFolder);
    if (configPath) {
      args.push("--config", configPath);
    }
    args.push(tempFile);

    const { stdout } = await execFile(command, args, { cwd }, token);
    return stdout;
  } finally {
    await fs.rm(tempDir, { recursive: true, force: true });
  }
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

function execFile(command, args, options, token) {
  return new Promise((resolve, reject) => {
    const child = childProcess.execFile(
      command,
      args,
      {
        ...options,
        maxBuffer: 16 * 1024 * 1024,
      },
      (error, stdout, stderr) => {
        if (error) {
          const detail = stderr.trim() || error.message;
          reject(new Error(`erbfmt failed: ${detail}`));
          return;
        }

        resolve({ stdout, stderr });
      },
    );

    token.onCancellationRequested(() => {
      child.kill();
      reject(new Error("erbfmt formatting was cancelled."));
    });
  });
}

module.exports = {
  activate,
  deactivate,
};
