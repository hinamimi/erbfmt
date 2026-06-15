const ERB_TAG_PATTERN = /<%[\s\S]*?%>/g;

export function toggleErbCommentLine(line: string): string {
  const match = line.match(/^(\s*)(.*)$/);
  const indent = match?.[1] ?? "";
  const content = match?.[2] ?? line;

  return `${indent}${toggleErbCommentContent(content)}`;
}

function toggleErbCommentContent(content: string): string {
  const uncommented = uncommentContent(content);
  if (uncommented !== undefined) {
    return uncommented;
  }

  return commentContent(content);
}

function uncommentContent(content: string): string | undefined {
  let cursor = 0;
  let output = "";
  let sawComment = false;

  while (cursor < content.length) {
    if (content.startsWith("<%#", cursor)) {
      const end = content.indexOf("%>", cursor + "<%#".length);
      if (end === -1) {
        return undefined;
      }

      const isOutput = content[cursor + "<%#".length] === "=";
      const bodyStart = cursor + (isOutput ? "<%#=".length : "<%#".length);
      const body = content.slice(bodyStart, end).trim();
      output += isOutput ? `<%= ${body} %>` : `<% ${body} %>`;
      cursor = end + "%>".length;
      sawComment = true;
      continue;
    }

    if (content.startsWith("<!--", cursor)) {
      const end = content.indexOf("-->", cursor + "<!--".length);
      if (end === -1) {
        return undefined;
      }

      output += content.slice(cursor + "<!--".length, end);
      cursor = end + "-->".length;
      sawComment = true;
      continue;
    }

    if (content[cursor]?.trim() === "") {
      output += content[cursor];
      cursor += 1;
      continue;
    }

    return undefined;
  }

  return sawComment ? output : undefined;
}

function commentContent(content: string): string {
  if (content.trim() === "") {
    return "<!---->";
  }

  const singleErbComment = commentErbTag(content.trim());
  if (singleErbComment) {
    return singleErbComment;
  }

  let cursor = 0;
  let output = "";

  for (const match of content.matchAll(ERB_TAG_PATTERN)) {
    const start = match.index;
    const tag = match[0];

    if (start > cursor) {
      output += commentHtmlFragment(content.slice(cursor, start));
    }

    output += commentErbTag(tag) ?? commentHtmlFragment(tag);
    cursor = start + tag.length;
  }

  if (cursor < content.length) {
    output += commentHtmlFragment(content.slice(cursor));
  }

  return output || commentHtmlFragment(content);
}

function commentErbTag(tag: string): string | undefined {
  const outputMatch = tag.match(/^<%=\s*([\s\S]*?)\s*%>$/);
  if (outputMatch) {
    return `<%#= ${outputMatch[1].trim()} %>`;
  }

  const codeMatch = tag.match(/^<%(?!#|=)\s*([\s\S]*?)\s*%>$/);
  if (codeMatch) {
    return `<%# ${codeMatch[1].trim()} %>`;
  }

  return undefined;
}

function commentHtmlFragment(fragment: string): string {
  return fragment ? `<!--${fragment}-->` : "";
}
