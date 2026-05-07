import test from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";

const messageHeaderPath = new URL("./MessageHeader.svelte", import.meta.url);

test("new and existing recipients share the same title typography class", async () => {
  const source = await readFile(messageHeaderPath, "utf8");

  assert.match(source, /const recipientTitleClass\s*=\s*/);
  assert.match(
    source,
    /class={`\$\{recipientTitleClass\} \$\{\s*concatInputText \? "" : "italic"\s*\}`}/,
  );
  assert.match(source, /class={recipientTitleClass}/);
});
