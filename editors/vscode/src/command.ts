export type ParsedCommandSetting = {
  command: string;
  arguments: string[];
};

export function parseCommandSetting(value: string): ParsedCommandSetting {
  const parts = splitCommandSetting(value);
  const [command = "erbfmt", ...args] = parts;

  return {
    command,
    arguments: args,
  };
}

export function splitCommandSetting(value: string): string[] {
  const parts: string[] = [];
  let current = "";
  let quote: '"' | "'" | undefined;
  let escaped = false;

  for (const character of value) {
    if (escaped) {
      current += character;
      escaped = false;
      continue;
    }

    if (character === "\\") {
      escaped = true;
      continue;
    }

    if (quote) {
      if (character === quote) {
        quote = undefined;
      } else {
        current += character;
      }
      continue;
    }

    if (character === '"' || character === "'") {
      quote = character;
      continue;
    }

    if (/\s/.test(character)) {
      if (current) {
        parts.push(current);
        current = "";
      }
      continue;
    }

    current += character;
  }

  if (escaped) {
    current += "\\";
  }

  if (current) {
    parts.push(current);
  }

  return parts;
}
