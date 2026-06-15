const assert = require("node:assert");
const { toggleErbCommentLine } = require("../out/comment");

const examples = [
  ["  <% elsif guest? %>", "  <%# elsif guest? %>"],
  ["  <%= user.name %>", "  <%#= user.name %>"],
  ["    <p>Please sign in.</p>", "    <!--<p>Please sign in.</p>-->"],
  ["<p><%= user.name %></p>", "<!--<p>--><%#= user.name %><!--</p>-->"],
  ["", "<!---->"],
];

for (const [input, commented] of examples) {
  assert.strictEqual(toggleErbCommentLine(input), commented);
  assert.strictEqual(toggleErbCommentLine(commented), input);
}

console.log("VSCode comment toggle test passed.");
