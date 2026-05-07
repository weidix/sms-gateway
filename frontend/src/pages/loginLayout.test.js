import test from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";

const loginPagePath = new URL("./Login.svelte", import.meta.url);

test("compact login layout hides the decorative marketing column", async () => {
  const source = await readFile(loginPagePath, "utf8");

  assert.match(source, /<section class="[^"]*hidden[^"]*lg:flex[^"]*"/);
  assert.doesNotMatch(source, /md:flex/);
});

test("compact login layout keeps the form vertically centered", async () => {
  const source = await readFile(loginPagePath, "utf8");

  assert.match(
    source,
    /<div class="mx-auto flex min-h-\[calc\(100dvh-5\.5rem\)\] w-full max-w-7xl items-center">/,
  );
});
