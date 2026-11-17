const assert = require("node:assert/strict");
const { formattingEdits } = require("../out/formatEdits");

for (const [before, after] of [
  ["", "hello\n"],
  ["hello\n", ""],
  ["hello", "hello\n"],
  ["hello\n", "hello"],
  ["a\r\nb\r\n", "a\nb\n"],
  ["a\nb\nc\n", "a\nc\n"],
  ["a\nc\n", "a\nb\nc\n"],
  ["same\nsame\nx\n", "same\nx\nsame\n"],
  ["<p>日本語😀</p>", "<p>日本語😀</p>\n"],
]) {
  let result = before;
  for (const edit of formattingEdits(before, after).reverse()) {
    result = result.slice(0, edit.start) + edit.replacement + result.slice(edit.end);
  }
  assert.equal(result, after);
}

const before = "a\nwrong\nunchanged\nwrong\nz\n";
const after = "a\nright\nunchanged\nright\nz\n";
const edits = formattingEdits(before, after);
assert.equal(edits.length, 2);
assert.equal(
  before.slice(0, edits[0].start) + edits[0].replacement + before.slice(edits[0].end),
  "a\nright\nunchanged\nwrong\nz\n",
);
assert.deepEqual(formattingEdits(after, after), []);
console.log("Formatting edit tests passed.");

// Exercise resynchronization beyond its lookahead, repeated lines, adjacent
// changes, CRLF, and EOF changes without permitting overlapping edits.
const samples = [
  "",
  "a",
  "a\n",
  "a\r\n",
  "a\nb\nc\n",
  "a\na\na\n",
  "日本語😀\r\nend",
  `${"same\n".repeat(30)}last\n`,
];
for (const source of samples) {
  for (const formatted of samples) {
    const changes = formattingEdits(source, formatted);
    for (let index = 1; index < changes.length; index += 1) {
      assert(changes[index - 1].end <= changes[index].start);
    }
    let applied = source;
    for (const edit of [...changes].reverse()) {
      applied = applied.slice(0, edit.start) + edit.replacement + applied.slice(edit.end);
    }
    assert.equal(applied, formatted);
  }
}
