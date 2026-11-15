export type FormatEdit = { start: number; end: number; replacement: string };

// Keep line endings in the chunks so edits also cover CRLF and the final newline.
export function formattingEdits(current: string, formatted: string): FormatEdit[] {
  const before = current.match(/[^\n]*\n|[^\n]+$/g) ?? [];
  const after = formatted.match(/[^\n]*\n|[^\n]+$/g) ?? [];
  const edits: FormatEdit[] = [];
  let left = 0;
  let right = 0;
  let offset = 0;
  while (left < before.length || right < after.length) {
    if (left < before.length && before[left] === after[right]) {
      offset += before[left].length;
      left += 1;
      right += 1;
      continue;
    }
    const start = offset;
    const replacement: string[] = [];
    while (left < before.length || right < after.length) {
      if (left < before.length && before[left] === after[right]) break;
      let sync: { left: number; right: number; distance: number } | undefined;
      for (let i = left; i < Math.min(before.length, left + 20); i += 1) {
        for (let j = right; j < Math.min(after.length, right + 20); j += 1) {
          const distance = i - left + j - right;
          if (before[i] === after[j] && (!sync || distance < sync.distance)) {
            sync = { left: i, right: j, distance };
          }
        }
      }
      const nextLeft = sync?.left ?? Math.min(left + 1, before.length);
      const nextRight = sync?.right ?? Math.min(right + 1, after.length);
      while (left < nextLeft) offset += before[left++].length;
      while (right < nextRight) replacement.push(after[right++]);
    }
    edits.push({ start, end: offset, replacement: replacement.join("") });
  }
  return edits;
}
