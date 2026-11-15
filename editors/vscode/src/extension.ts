import * as childProcess from "node:child_process";
import { constants as fsConstants } from "node:fs";
import * as fs from "node:fs/promises";
import * as os from "node:os";
import * as path from "node:path";
import * as vscode from "vscode";
import { parseCommandSetting } from "./command";
import { toggleErbCommentLine } from "./comment";
import { formattingEdits } from "./formatEdits";

const diagnosticVersions = new WeakMap<vscode.Diagnostic, number>();
const diagnosticIgnoreEdits = new WeakMap<vscode.Diagnostic, CliTextEdit>();
type CliTextEdit = { range: { start: number; end: number }; replacement: string };

const FORMATTER_SELECTOR: vscode.DocumentFilter[] = [
  { scheme: "file", language: "erb" },
  { scheme: "file", language: "html-erb" },
];

type CommandContext = {
  command: string;
  cwd: string;
  extraArguments: string[];
  resolution: CommandResolution;
  checkoutRoot: string | undefined;
  localBinary: string | undefined;
  settings: vscode.WorkspaceConfiguration;
  workspaceFolder: vscode.WorkspaceFolder | undefined;
};

type CommandResolution =
  | "configured"
  | "checkoutBinary"
  | "checkoutCargoFallback"
  | "pathCargo"
  | "pathErbfmt";

type ExecResult = {
  stdout: string;
  stderr: string;
  exitCode: number | string;
  errorMessage: string | undefined;
};

type ExecOptions = {
  cwd: string;
  input: string;
};

export function activate(context: vscode.ExtensionContext): void {
  const diagnostics = vscode.languages.createDiagnosticCollection("erbfmt");
  const output = vscode.window.createOutputChannel("erbfmt");
  const pending = new Map<string, ReturnType<typeof setTimeout>>();
  const running = new Map<string, vscode.CancellationTokenSource>();
  const refresh = async (document: vscode.TextDocument, delay = 0): Promise<void> => {
    const key = document.uri.toString();
    clearTimeout(pending.get(key));
    running.get(key)?.cancel();
    running.get(key)?.dispose();
    if (!isSupportedDocument(document)) return;
    const cancellation = new vscode.CancellationTokenSource();
    running.set(key, cancellation);
    pending.delete(key);
    if (delay > 0) {
      pending.set(
        key,
        setTimeout(() => {
          pending.delete(key);
          void refreshDiagnostics(document, diagnostics, cancellation.token);
        }, delay),
      );
    } else {
      await refreshDiagnostics(document, diagnostics, cancellation.token);
    }
  };
  context.subscriptions.push({
    dispose() {
      for (const timer of pending.values()) clearTimeout(timer);
      for (const cancellation of running.values()) {
        cancellation.cancel();
        cancellation.dispose();
      }
    },
  });
  const provider: vscode.DocumentFormattingEditProvider = {
    async provideDocumentFormattingEdits(document, _options, token) {
      const version = document.version;
      const formatted = await formatDocument(document, token);

      return document.version !== version || formatted === document.getText()
        ? []
        : [vscode.TextEdit.replace(fullDocumentRange(document), formatted)];
    },
  };

  context.subscriptions.push(
    diagnostics,
    output,
    vscode.languages.registerDocumentFormattingEditProvider(FORMATTER_SELECTOR, provider),
    vscode.languages.registerCodeActionsProvider(
      FORMATTER_SELECTOR,
      { provideCodeActions },
      { providedCodeActionKinds: [vscode.CodeActionKind.QuickFix] },
    ),
    vscode.commands.registerCommand("erbfmt.formatDocument", async () => {
      const editor = vscode.window.activeTextEditor;
      if (!editor) {
        return;
      }

      try {
        const version = editor.document.version;
        const formatted = await formatDocument(editor.document, createNullToken());
        if (editor.document.version !== version || formatted === editor.document.getText()) {
          return;
        }
        await editor.edit((edit) => {
          edit.replace(fullDocumentRange(editor.document), formatted);
        });
      } catch (error) {
        await vscode.window.showErrorMessage(errorMessage(error));
      }
    }),
    vscode.commands.registerCommand(
      "erbfmt.applyQuickFix",
      async (
        uri: vscode.Uri,
        version: number,
        edit: { start: number; end: number; replacement: string },
      ) => {
        const document = await vscode.workspace.openTextDocument(uri);
        if (
          !isSupportedDocument(document) ||
          !vscode.workspace.isTrusted ||
          document.version !== version ||
          document.isClosed
        )
          return false;
        const workspaceEdit = new vscode.WorkspaceEdit();
        workspaceEdit.replace(
          uri,
          new vscode.Range(document.positionAt(edit.start), document.positionAt(edit.end)),
          edit.replacement,
        );
        const applied = await vscode.workspace.applyEdit(workspaceEdit);
        if (applied) await refresh(document);
        return applied;
      },
    ),
    vscode.commands.registerCommand("erbfmt.lintDocument", async (uri?: vscode.Uri) => {
      const document = uri
        ? await vscode.workspace.openTextDocument(uri)
        : vscode.window.activeTextEditor?.document;
      if (!document) return;
      await refresh(document);
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
        output.appendLine(`resolution: ${commandResolutionLabel(commandInfo.resolution)}`);
        output.appendLine(`command: ${commandInfo.commandLine}`);
        output.appendLine(`cwd: ${commandInfo.cwd}`);
        if (commandInfo.checkoutRoot) {
          output.appendLine(`checkout: ${commandInfo.checkoutRoot}`);
        }
        if (commandInfo.localBinary) {
          output.appendLine(`checkout binary: ${commandInfo.localBinary}`);
        }
        if (commandInfo.configPath) {
          output.appendLine(`config: ${commandInfo.configPath}`);
        } else {
          output.appendLine("config: <none>");
        }
        if (commandInfo.setupHint) {
          output.appendLine("");
          output.appendLine(commandInfo.setupHint);
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
      void refresh(document);
    }),
    vscode.workspace.onDidSaveTextDocument((document) => {
      void refresh(document);
    }),
    vscode.workspace.onDidChangeTextDocument(({ document }) => {
      diagnostics.delete(document.uri);
      void refresh(document, 300);
    }),
    vscode.workspace.onDidCloseTextDocument((document) => {
      const key = document.uri.toString();
      clearTimeout(pending.get(key));
      pending.delete(key);
      running.get(key)?.cancel();
      running.get(key)?.dispose();
      running.delete(key);
      diagnostics.delete(document.uri);
    }),
  );

  for (const document of vscode.workspace.textDocuments) {
    void refresh(document);
  }
}

type AssistAction = {
  id: string;
  rule: string;
  title: string;
  range: { start: number; end: number };
  edit: CliTextEdit;
};

async function provideCodeActions(
  document: vscode.TextDocument,
  range: vscode.Range,
  context: vscode.CodeActionContext,
  token: vscode.CancellationToken,
): Promise<vscode.CodeAction[]> {
  if (!isSupportedDocument(document) || !vscode.workspace.isTrusted) return [];
  const version = document.version;
  const source = document.getText();
  const relevant = context.diagnostics.filter(
    (diagnostic) =>
      diagnostic.source === "erbfmt" &&
      diagnosticVersions.get(diagnostic) === version &&
      diagnostic.code,
  );
  if (relevant.length === 0) return [];
  const actions: vscode.CodeAction[] = [];
  const bytes = Buffer.from(source);
  const bytePosition = (offset: number) =>
    document.positionAt(bytes.subarray(0, offset).toString("utf8").length);
  const actionForEdit = (
    title: string,
    editRange: vscode.Range,
    replacement: string,
    related: vscode.Diagnostic[],
    preferred = false,
  ) => {
    const action = new vscode.CodeAction(title, vscode.CodeActionKind.QuickFix);
    action.diagnostics = related;
    action.isPreferred = preferred;
    action.command = {
      command: "erbfmt.applyQuickFix",
      title,
      arguments: [
        document.uri,
        version,
        {
          start: document.offsetAt(editRange.start),
          end: document.offsetAt(editRange.end),
          replacement,
        },
      ],
    };
    actions.push(action);
  };
  if (relevant.some((diagnostic) => diagnostic.code === "format")) {
    try {
      const formatted = await formatDocument(document, token);
      for (const edit of formattingEdits(source, formatted)) {
        const editRange = new vscode.Range(
          document.positionAt(edit.start),
          document.positionAt(edit.end),
        );
        const related = relevant.filter(
          (diagnostic) => diagnostic.code === "format" && diagnostic.range.isEqual(editRange),
        );
        if (related.length > 0)
          actionForEdit("erbfmt: Format this change", editRange, edit.replacement, related, true);
      }
    } catch {
      /* Execution failures are reported through diagnostics. */
    }
  }
  const ignoreKeys = new Set<string>();
  for (const diagnostic of relevant) {
    const edit = diagnosticIgnoreEdits.get(diagnostic);
    const key = `${diagnostic.code}:${edit?.range.start}`;
    if (edit && !ignoreKeys.has(key)) {
      ignoreKeys.add(key);
      actionForEdit(
        `erbfmt: Ignore ${diagnostic.code} on this line`,
        new vscode.Range(bytePosition(edit.range.start), bytePosition(edit.range.end)),
        edit.replacement,
        relevant.filter(
          (item) =>
            item.code === diagnostic.code &&
            diagnosticIgnoreEdits.get(item)?.range.start === edit.range.start,
        ),
      );
    }
  }
  try {
    const command = await getCommandContext(document);
    const args = await buildErbfmtArgs(command, document, [
      "--assist",
      "--assist-format",
      "json",
      "--stdin-filepath",
      document.uri.fsPath,
    ]);
    const result = await execFile(
      command.command,
      args,
      { cwd: command.cwd, input: source },
      token,
    );
    if (result.exitCode === 0 && result.stdout.trim()) {
      const payload = JSON.parse(result.stdout) as {
        actions: AssistAction[];
        ignoreTargets?: {
          line: number;
          range: { start: number; end: number };
          format: CliTextEdit | null;
        }[];
      };
      for (const assist of payload.actions) {
        const target = new vscode.Range(
          bytePosition(assist.range.start),
          bytePosition(assist.range.end),
        );
        const related = relevant.filter(
          (diagnostic) => diagnostic.code === assist.rule && target.intersection(diagnostic.range),
        );
        if (related.length > 0 && target.intersection(range))
          actionForEdit(
            `erbfmt: ${assist.title}`,
            new vscode.Range(
              bytePosition(assist.edit.range.start),
              bytePosition(assist.edit.range.end),
            ),
            assist.edit.replacement,
            related,
            true,
          );
      }
      const seen = new Set<number>();
      for (const diagnostic of relevant.filter((item) => item.code === "format")) {
        const lastLine =
          diagnostic.range.end.line -
          Number(
            diagnostic.range.end.character === 0 &&
              diagnostic.range.end.line > diagnostic.range.start.line,
          );
        const target = payload.ignoreTargets?.find(
          (item) =>
            item.format &&
            item.line === diagnostic.range.start.line + 1 &&
            bytePosition(item.range.end).line >= lastLine,
        );
        if (target?.format && !seen.has(target.line)) {
          seen.add(target.line);
          actionForEdit(
            "erbfmt: Ignore formatting for the next node",
            new vscode.Range(
              bytePosition(target.format.range.start),
              bytePosition(target.format.range.end),
            ),
            target.format.replacement,
            [diagnostic],
          );
        }
      }
    }
  } catch {
    /* Older CLI versions may not support assist output. */
  }
  return document.version === version && !token.isCancellationRequested ? actions : [];
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

  const args = await buildErbfmtArgs(context, document, ["--stdin-filepath", document.uri.fsPath]);
  const result = await execFile(
    context.command,
    args,
    { cwd: context.cwd, input: document.getText() },
    token,
  );
  if (result.exitCode !== 0) {
    throw new Error(formatFailureMessage(context, args, result));
  }

  return result.stdout;
}

async function refreshDiagnostics(
  document: vscode.TextDocument,
  diagnostics: vscode.DiagnosticCollection,
  token: vscode.CancellationToken,
): Promise<void> {
  if (!isSupportedDocument(document)) {
    diagnostics.delete(document.uri);
    return;
  }

  const settings = vscode.workspace.getConfiguration("erbfmt", document.uri);
  const lintEnabled = settings.get("lint.enabled", true);
  const formatDiagnosticsEnabled = settings.get("formatDiagnostics.enabled", true);

  if (!lintEnabled && !formatDiagnosticsEnabled) {
    diagnostics.delete(document.uri);
    return;
  }

  const version = document.version;
  const nextDiagnostics: vscode.Diagnostic[] = [];

  if (lintEnabled) {
    nextDiagnostics.push(...(await lintDocument(document, token)));
  }

  if (formatDiagnosticsEnabled) {
    nextDiagnostics.push(
      ...(await collectFormatDiagnostics(document, token, { reportExecutionErrors: !lintEnabled })),
    );
  }

  if (document.version !== version || document.isClosed || token.isCancellationRequested) return;
  for (const diagnostic of nextDiagnostics) diagnosticVersions.set(diagnostic, version);
  if (nextDiagnostics.length > 0) {
    diagnostics.set(document.uri, nextDiagnostics);
  } else {
    diagnostics.delete(document.uri);
  }
}

async function lintDocument(
  document: vscode.TextDocument,
  token: vscode.CancellationToken,
): Promise<vscode.Diagnostic[]> {
  try {
    const context = await getCommandContext(document);
    const args = await buildErbfmtArgs(context, document, [
      "--lint",
      "--lint-format",
      "json",
      "--stdin-filepath",
      document.uri.fsPath,
    ]);
    const result = await execFile(
      context.command,
      args,
      { cwd: context.cwd, input: document.getText() },
      token,
    );
    if (!result.stdout.trim()) {
      if (result.exitCode === 0) return [];
      throw new Error(formatFailureMessage(context, args, result));
    }
    const payload = JSON.parse(result.stdout);
    const parsedDiagnostics = payload.diagnostics.map(
      (item: {
        message: string;
        severity: string;
        rule?: string | null;
        ignore?: CliTextEdit | null;
        location: { line: number; column: number } | null;
      }) => {
        const message = item.location
          ? `${item.message} at line ${item.location.line}, column ${item.location.column}`
          : item.message;
        const diagnostic = createDiagnostic(
          rangeFromMessage(document, message),
          message,
          item.severity === "info"
            ? vscode.DiagnosticSeverity.Information
            : item.severity === "warning"
              ? vscode.DiagnosticSeverity.Warning
              : vscode.DiagnosticSeverity.Error,
        );
        diagnostic.code = item.rule ?? undefined;
        if (item.ignore) diagnosticIgnoreEdits.set(diagnostic, item.ignore);
        return diagnostic;
      },
    );

    if (parsedDiagnostics.length > 0) {
      return parsedDiagnostics;
    }

    if (result.exitCode === 0) {
      return [];
    }

    return [
      createDiagnostic(
        firstCharacterRange(document),
        formatFailureMessage(context, args, result),
        vscode.DiagnosticSeverity.Error,
      ),
    ];
  } catch (error) {
    return [
      createDiagnostic(
        firstCharacterRange(document),
        errorMessage(error),
        vscode.DiagnosticSeverity.Error,
      ),
    ];
  }
}

type FormatDiagnosticsOptions = {
  reportExecutionErrors: boolean;
};

async function collectFormatDiagnostics(
  document: vscode.TextDocument,
  token: vscode.CancellationToken,
  options: FormatDiagnosticsOptions,
): Promise<vscode.Diagnostic[]> {
  if (token.isCancellationRequested) {
    return [];
  }

  try {
    const context = await getCommandContext(document);
    const args = await buildErbfmtArgs(context, document, [
      "--stdin-filepath",
      document.uri.fsPath,
    ]);
    const result = await execFile(
      context.command,
      args,
      { cwd: context.cwd, input: document.getText() },
      token,
    );

    if (result.exitCode !== 0) {
      if (!options.reportExecutionErrors) {
        return [];
      }

      return [
        createDiagnostic(
          firstCharacterRange(document),
          formatFailureMessage(context, args, result),
          vscode.DiagnosticSeverity.Error,
        ),
      ];
    }

    return formatDiagnostics(document, result.stdout);
  } catch (error) {
    if (!options.reportExecutionErrors) {
      return [];
    }

    return [
      createDiagnostic(
        firstCharacterRange(document),
        errorMessage(error),
        vscode.DiagnosticSeverity.Error,
      ),
    ];
  }
}

type CommandDescription = {
  commandLine: string;
  cwd: string;
  configPath: string | undefined;
  resolution: CommandResolution;
  checkoutRoot: string | undefined;
  localBinary: string | undefined;
  setupHint: string | undefined;
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
    resolution: context.resolution,
    checkoutRoot: context.checkoutRoot,
    localBinary: context.localBinary,
    setupHint: commandContextSetupHint(context),
  };
}

async function getCommandContext(document: vscode.TextDocument): Promise<CommandContext> {
  const workspaceFolder = vscode.workspace.getWorkspaceFolder(document.uri);
  let cwd = workspaceFolder?.uri.fsPath ?? path.dirname(document.uri.fsPath);
  const settings = vscode.workspace.getConfiguration("erbfmt", document.uri);
  const commandSetting = settings.get("command", "erbfmt").trim() || "erbfmt";
  const parsedCommand = parseCommandSetting(commandSetting);
  let command = parsedCommand.command;
  const configuredArguments = settings.get<unknown>("arguments", []);
  let extraArguments = Array.isArray(configuredArguments)
    ? [
        ...parsedCommand.arguments,
        ...configuredArguments.filter(
          (argument): argument is string => typeof argument === "string",
        ),
      ]
    : parsedCommand.arguments;
  let resolution: CommandResolution = "configured";
  let checkoutRoot: string | undefined;
  let localBinary: string | undefined;
  const inspectedCommand = settings.inspect<string>("command");
  const commandIsDefault =
    inspectedCommand &&
    inspectedCommand.globalValue === undefined &&
    inspectedCommand.workspaceValue === undefined &&
    inspectedCommand.workspaceFolderValue === undefined &&
    inspectedCommand.defaultValue === commandSetting;

  checkoutRoot = await findNearestErbfmtCheckout(cwd);
  if (commandIsDefault && checkoutRoot) {
    localBinary = path.join(checkoutRoot, "target", "debug", "erbfmt");
    if (await isExecutableFile(localBinary)) {
      command = localBinary;
      resolution = "checkoutBinary";
    } else {
      command = await findCargoCommand();
      extraArguments = ["run", "--quiet", "--", ...extraArguments];
      resolution = "checkoutCargoFallback";
    }
    cwd = checkoutRoot;
  } else if (commandIsDefault) {
    resolution = "pathErbfmt";
  }

  if (command === "cargo") {
    command = await findCargoCommand();
    if (resolution === "configured") {
      resolution = "pathCargo";
    }
  }

  return {
    command,
    cwd,
    extraArguments,
    resolution,
    checkoutRoot,
    localBinary,
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

function formatDiagnostics(document: vscode.TextDocument, formatted: string): vscode.Diagnostic[] {
  return formattingEdits(document.getText(), formatted)
    .slice(0, 100)
    .map((edit) => {
      const diagnostic = createDiagnostic(
        new vscode.Range(document.positionAt(edit.start), document.positionAt(edit.end)),
        "File is not formatted. Run Format Document to apply erbfmt.",
        vscode.DiagnosticSeverity.Warning,
      );
      diagnostic.code = "format";
      return diagnostic;
    });
}

function rangeFromMessage(document: vscode.TextDocument, message: string): vscode.Range {
  const match = message.match(/ at line (\d+), column (\d+)$/);
  if (!match) {
    return firstCharacterRange(document);
  }

  const line = Math.max(0, Math.min(Number(match[1]) - 1, document.lineCount - 1));
  const lineText = document.lineAt(line).text;
  const column = Array.from(lineText)
    .slice(0, Math.max(0, Number(match[2]) - 1))
    .join("").length;
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

function createDiagnostic(
  range: vscode.Range,
  message: string,
  severity: vscode.DiagnosticSeverity,
): vscode.Diagnostic {
  const diagnostic = new vscode.Diagnostic(range, message, severity);
  diagnostic.source = "erbfmt";

  return diagnostic;
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
    if (token.isCancellationRequested) {
      resolve({
        stdout: "",
        stderr: "erbfmt formatting was cancelled.",
        exitCode: 1,
        errorMessage: undefined,
      });
      return;
    }
    let cancellation: vscode.Disposable | undefined;
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
        cancellation?.dispose();
        resolve({
          stdout: stdout.toString(),
          stderr: stderr.toString(),
          exitCode: error?.code ?? 0,
          errorMessage: error?.message,
        });
      },
    );

    // A command that exits before reading stdin may close the pipe early.
    child.stdin?.on("error", () => {});
    child.stdin?.end(options.input, "utf8");

    cancellation = token.onCancellationRequested(() => {
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

function commandContextSetupHint(context: CommandContext): string | undefined {
  if (context.resolution === "checkoutCargoFallback") {
    return "Hint: run `cargo build` in the erbfmt checkout to use `target/debug/erbfmt` directly instead of `cargo run --quiet --`.";
  }

  if (context.resolution === "pathErbfmt") {
    return "Hint: using `erbfmt` from PATH. Set `erbfmt.command` to an absolute binary path for a pinned local setup.";
  }

  if (context.resolution === "pathCargo") {
    return "Hint: using cargo from PATH or ~/.cargo/bin. Prefer `cargo build` plus the checkout binary for local extension development.";
  }

  return undefined;
}

function commandResolutionLabel(resolution: CommandResolution): string {
  switch (resolution) {
    case "configured":
      return "configured command";
    case "checkoutBinary":
      return "checkout target/debug/erbfmt";
    case "checkoutCargoFallback":
      return "checkout cargo fallback";
    case "pathCargo":
      return "cargo from PATH";
    case "pathErbfmt":
      return "erbfmt from PATH";
  }
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
