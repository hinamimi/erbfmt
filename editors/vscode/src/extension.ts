import * as childProcess from "node:child_process";
import { constants as fsConstants } from "node:fs";
import * as fs from "node:fs/promises";
import * as os from "node:os";
import * as path from "node:path";
import * as vscode from "vscode";
import { toggleErbCommentLine } from "./comment";

const FORMATTER_SELECTOR: vscode.DocumentFilter[] = [
  { scheme: "file", language: "erb" },
  { scheme: "file", language: "html-erb" },
];

type CommandContext = {
  command: string;
  cwd: string;
  extraArguments: string[];
  settings: vscode.WorkspaceConfiguration;
  workspaceFolder: vscode.WorkspaceFolder | undefined;
};

type ExecResult = {
  stdout: string;
  stderr: string;
  exitCode: number | string;
  errorMessage: string | undefined;
};

type ExecOptions = {
  cwd: string;
};

export function activate(context: vscode.ExtensionContext): void {
  const diagnostics = vscode.languages.createDiagnosticCollection("erbfmt");
  const output = vscode.window.createOutputChannel("erbfmt");
  const provider: vscode.DocumentFormattingEditProvider = {
    async provideDocumentFormattingEdits(document, _options, token) {
      const formatted = await formatDocument(document, token);

      return [vscode.TextEdit.replace(fullDocumentRange(document), formatted)];
    },
  };

  context.subscriptions.push(
    diagnostics,
    output,
    vscode.languages.registerDocumentFormattingEditProvider(FORMATTER_SELECTOR, provider),
    vscode.commands.registerCommand("erbfmt.formatDocument", async () => {
      const editor = vscode.window.activeTextEditor;
      if (!editor) {
        return;
      }

      try {
        const formatted = await formatDocument(editor.document, createNullToken());
        await editor.edit((edit) => {
          edit.replace(fullDocumentRange(editor.document), formatted);
        });
      } catch (error) {
        await vscode.window.showErrorMessage(errorMessage(error));
      }
    }),
    vscode.commands.registerCommand("erbfmt.lintDocument", async () => {
      const editor = vscode.window.activeTextEditor;
      if (!editor) {
        return;
      }

      await lintDocument(editor.document, diagnostics, createNullToken());
    }),
    vscode.commands.registerCommand("erbfmt.showCommand", async () => {
      const editor = vscode.window.activeTextEditor;
      if (!editor) {
        await vscode.window.showWarningMessage(
          "Open an ERB document before showing erbfmt command.",
        );
        return;
      }

      try {
        const commandInfo = await describeCommand(editor.document);
        output.clear();
        output.appendLine("erbfmt command resolution");
        output.appendLine("");
        output.appendLine(`command: ${commandInfo.commandLine}`);
        output.appendLine(`cwd: ${commandInfo.cwd}`);
        if (commandInfo.configPath) {
          output.appendLine(`config: ${commandInfo.configPath}`);
        } else {
          output.appendLine("config: <none>");
        }
        output.show(true);
      } catch (error) {
        await vscode.window.showErrorMessage(errorMessage(error));
      }
    }),
    vscode.commands.registerCommand("erbfmt.toggleComment", async () => {
      const editor = vscode.window.activeTextEditor;
      if (!editor) {
        return;
      }

      if (!isSupportedDocument(editor.document)) {
        await vscode.commands.executeCommand("editor.action.commentLine");
        return;
      }

      await toggleEditorComment(editor);
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

export function deactivate(): void {}

async function toggleEditorComment(editor: vscode.TextEditor): Promise<void> {
  const ranges = selectedLineRanges(editor.document, editor.selections);

  await editor.edit((edit) => {
    for (const range of ranges) {
      for (let line = range.start; line <= range.end; line += 1) {
        const documentLine = editor.document.lineAt(line);
        edit.replace(documentLine.range, toggleErbCommentLine(documentLine.text));
      }
    }
  });
}

type SelectedLineRange = {
  start: number;
  end: number;
};

function selectedLineRanges(
  document: vscode.TextDocument,
  selections: readonly vscode.Selection[],
): SelectedLineRange[] {
  const ranges = selections
    .map((selection) => {
      const start = selection.start.line;
      let end = selection.end.line;

      if (!selection.isEmpty && selection.end.character === 0 && end > start) {
        end -= 1;
      }

      return {
        start: Math.max(0, Math.min(start, document.lineCount - 1)),
        end: Math.max(0, Math.min(end, document.lineCount - 1)),
      };
    })
    .sort((left, right) => left.start - right.start || left.end - right.end);

  const merged: SelectedLineRange[] = [];
  for (const range of ranges) {
    const previous = merged.at(-1);
    if (previous && range.start <= previous.end + 1) {
      previous.end = Math.max(previous.end, range.end);
    } else {
      merged.push({ ...range });
    }
  }

  return merged;
}

async function formatDocument(
  document: vscode.TextDocument,
  token: vscode.CancellationToken,
): Promise<string> {
  if (document.uri.scheme !== "file") {
    throw new Error("erbfmt can only format files on disk.");
  }

  if (token.isCancellationRequested) {
    return document.getText();
  }

  const context = await getCommandContext(document);

  const tempDir = await fs.mkdtemp(path.join(os.tmpdir(), "erbfmt-vscode-"));
  const tempFile = path.join(tempDir, path.basename(document.uri.fsPath));

  try {
    await fs.writeFile(tempFile, document.getText(), "utf8");

    const args = await buildErbfmtArgs(context, document, [tempFile]);

    const result = await execFile(context.command, args, { cwd: context.cwd }, token);
    if (result.exitCode !== 0) {
      throw new Error(formatFailureMessage(context, args, result));
    }

    return result.stdout;
  } finally {
    await fs.rm(tempDir, { recursive: true, force: true });
  }
}

async function lintDocument(
  document: vscode.TextDocument,
  diagnostics: vscode.DiagnosticCollection,
  token: vscode.CancellationToken,
): Promise<void> {
  if (!isSupportedDocument(document)) {
    diagnostics.delete(document.uri);
    return;
  }

  const settings = vscode.workspace.getConfiguration("erbfmt", document.uri);
  if (!settings.get("lint.enabled", true)) {
    diagnostics.delete(document.uri);
    return;
  }

  const context = await getCommandContext(document);
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

    const parsedDiagnostics = parseDiagnostics(document, tempFile, result.stderr);
    diagnostics.set(
      document.uri,
      parsedDiagnostics.length > 0
        ? parsedDiagnostics
        : [
            new vscode.Diagnostic(
              firstCharacterRange(document),
              formatFailureMessage(context, args, result),
              vscode.DiagnosticSeverity.Error,
            ),
          ],
    );
  } catch (error) {
    diagnostics.set(document.uri, [
      new vscode.Diagnostic(
        firstCharacterRange(document),
        errorMessage(error),
        vscode.DiagnosticSeverity.Error,
      ),
    ]);
  } finally {
    await fs.rm(tempDir, { recursive: true, force: true });
  }
}

type CommandDescription = {
  commandLine: string;
  cwd: string;
  configPath: string | undefined;
};

async function describeCommand(document: vscode.TextDocument): Promise<CommandDescription> {
  if (document.uri.scheme !== "file") {
    throw new Error("erbfmt can only inspect files on disk.");
  }

  const context = await getCommandContext(document);
  const configPath = await resolveConfigPath(context.settings, document, context.workspaceFolder);
  const args = [...context.extraArguments];

  if (configPath) {
    args.push("--config", configPath);
  }

  args.push("<file>");

  return {
    commandLine: commandLine(context.command, args),
    cwd: context.cwd,
    configPath,
  };
}

async function getCommandContext(document: vscode.TextDocument): Promise<CommandContext> {
  const workspaceFolder = vscode.workspace.getWorkspaceFolder(document.uri);
  let cwd = workspaceFolder?.uri.fsPath ?? path.dirname(document.uri.fsPath);
  const settings = vscode.workspace.getConfiguration("erbfmt", document.uri);
  let command = settings.get("command", "erbfmt");
  const configuredArguments = settings.get<unknown>("arguments", []);
  let extraArguments = Array.isArray(configuredArguments)
    ? configuredArguments.filter((argument): argument is string => typeof argument === "string")
    : [];
  const inspectedCommand = settings.inspect<string>("command");
  const commandIsDefault =
    inspectedCommand &&
    inspectedCommand.globalValue === undefined &&
    inspectedCommand.workspaceValue === undefined &&
    inspectedCommand.workspaceFolderValue === undefined &&
    inspectedCommand.defaultValue === command;

  const checkoutRoot = await findNearestErbfmtCheckout(cwd);
  if (commandIsDefault && checkoutRoot) {
    const localBinary = path.join(checkoutRoot, "target", "debug", "erbfmt");
    if (await isExecutableFile(localBinary)) {
      command = localBinary;
    } else {
      command = await findCargoCommand();
      extraArguments = ["run", "--quiet", "--", ...extraArguments];
    }
    cwd = checkoutRoot;
  }

  if (command === "cargo") {
    command = await findCargoCommand();
  }

  return {
    command,
    cwd,
    extraArguments,
    settings,
    workspaceFolder,
  };
}

async function isErbfmtCheckout(directory: string): Promise<boolean> {
  return (
    (await isFile(path.join(directory, "Cargo.toml"))) &&
    (await isFile(path.join(directory, "src", "main.rs"))) &&
    (await isFile(path.join(directory, "editors", "vscode", "package.json")))
  );
}

async function findNearestErbfmtCheckout(startDirectory: string): Promise<string | undefined> {
  let current = startDirectory;

  while (true) {
    if (await isErbfmtCheckout(current)) {
      return current;
    }

    const parent = path.dirname(current);
    if (parent === current) {
      return undefined;
    }

    current = parent;
  }
}

async function buildErbfmtArgs(
  context: CommandContext,
  document: vscode.TextDocument,
  trailingArguments: string[],
): Promise<string[]> {
  const args = [...context.extraArguments];
  const configPath = await resolveConfigPath(context.settings, document, context.workspaceFolder);

  if (configPath) {
    args.push("--config", configPath);
  }

  args.push(...trailingArguments);
  return args;
}

async function resolveConfigPath(
  settings: vscode.WorkspaceConfiguration,
  document: vscode.TextDocument,
  workspaceFolder: vscode.WorkspaceFolder | undefined,
): Promise<string | undefined> {
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

async function findNearestConfig(startDirectory: string): Promise<string | undefined> {
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

async function isFile(filePath: string): Promise<boolean> {
  try {
    const stat = await fs.stat(filePath);
    return stat.isFile();
  } catch (_error) {
    return false;
  }
}

async function isExecutableFile(filePath: string): Promise<boolean> {
  try {
    await fs.access(filePath, fsConstants.X_OK);
    return true;
  } catch (_error) {
    return false;
  }
}

async function findCargoCommand(): Promise<string> {
  const homeCargo = path.join(os.homedir(), ".cargo", "bin", "cargo");
  if (await isExecutableFile(homeCargo)) {
    return homeCargo;
  }

  return "cargo";
}

function parseDiagnostics(
  document: vscode.TextDocument,
  filePath: string,
  stderr: string,
): vscode.Diagnostic[] {
  const diagnostics: vscode.Diagnostic[] = [];

  for (const rawLine of stderr.split(/\r?\n/)) {
    const line = rawLine.trim();
    if (!line) {
      continue;
    }

    const message = line.startsWith(`${filePath}: `) ? line.slice(filePath.length + 2) : line;
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

function rangeFromMessage(document: vscode.TextDocument, message: string): vscode.Range {
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

function firstCharacterRange(document: vscode.TextDocument): vscode.Range {
  if (document.lineCount === 0) {
    return new vscode.Range(0, 0, 0, 0);
  }

  const firstLineLength = document.lineAt(0).text.length;
  return new vscode.Range(0, 0, 0, Math.min(1, firstLineLength));
}

function fullDocumentRange(document: vscode.TextDocument): vscode.Range {
  return new vscode.Range(document.positionAt(0), document.positionAt(document.getText().length));
}

function isSupportedDocument(document: vscode.TextDocument): boolean {
  return (
    document.uri.scheme === "file" &&
    FORMATTER_SELECTOR.some(
      (selector) =>
        typeof selector === "object" &&
        "language" in selector &&
        selector.language === document.languageId,
    )
  );
}

function createNullToken(): vscode.CancellationToken {
  const onCancellationRequested: vscode.Event<unknown> = () => ({
    dispose() {},
  });

  return {
    isCancellationRequested: false,
    onCancellationRequested,
  };
}

function execFile(
  command: string,
  args: string[],
  options: ExecOptions,
  token: vscode.CancellationToken,
): Promise<ExecResult> {
  return new Promise((resolve) => {
    const child = childProcess.execFile(
      command,
      args,
      {
        cwd: options.cwd,
        maxBuffer: 16 * 1024 * 1024,
      },
      (
        error: childProcess.ExecFileException | null,
        stdout: string | Buffer,
        stderr: string | Buffer,
      ) => {
        resolve({
          stdout: stdout.toString(),
          stderr: stderr.toString(),
          exitCode: error?.code ?? 0,
          errorMessage: error?.message,
        });
      },
    );

    token.onCancellationRequested(() => {
      child.kill();
      resolve({
        stdout: "",
        stderr: "erbfmt formatting was cancelled.",
        exitCode: 1,
        errorMessage: undefined,
      });
    });
  });
}

function formatFailureMessage(context: CommandContext, args: string[], result: ExecResult): string {
  const detail = result.stderr.trim() || result.errorMessage || `exit code ${result.exitCode}`;
  const hint = setupHint(context, result);

  return [
    `erbfmt failed: ${detail}`,
    hint,
    `command: ${commandLine(context.command, args)}`,
    `cwd: ${context.cwd}`,
  ]
    .filter(Boolean)
    .join("\n");
}

function setupHint(context: CommandContext, result: ExecResult): string | undefined {
  if (result.exitCode === "ENOENT") {
    if (path.basename(context.command) === "cargo") {
      return "Hint: VSCode could not find cargo. Run `cargo build` in the erbfmt checkout, install erbfmt, or set `erbfmt.command` to an absolute erbfmt binary path.";
    }

    return "Hint: erbfmt was not found. Run `cargo build` in this checkout, install erbfmt, or set `erbfmt.command` to an absolute erbfmt binary path.";
  }

  if (result.exitCode === "EACCES") {
    if (path.basename(context.command) === "cargo") {
      return "Hint: VSCode could not execute cargo. Run `cargo build` in the erbfmt checkout, install erbfmt, or set `erbfmt.command` to an executable erbfmt binary path.";
    }

    return "Hint: the configured erbfmt command is not executable. Check file permissions, run `cargo build`, or update `erbfmt.command`.";
  }

  return undefined;
}

function commandLine(command: string, args: string[]): string {
  return [command, ...args].map(shellQuote).join(" ");
}

function shellQuote(value: string): string {
  if (/^[A-Za-z0-9_./:=@+-]+$/.test(value)) {
    return value;
  }

  return `'${value.replaceAll("'", "'\\''")}'`;
}

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}
