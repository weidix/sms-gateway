import test from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";

const appPath = new URL("../../App.svelte", import.meta.url);
const messageListPath = new URL("./MessageList.svelte", import.meta.url);
const dashboardPagePath = new URL("../../pages/Dashboard.svelte", import.meta.url);
const conversationListPath = new URL("../conversation/ConversationList.svelte", import.meta.url);

test("app transition wrapper preserves a full-height message workspace", async () => {
  const source = await readFile(appPath, "utf8");

  assert.match(
    source,
    /<div[\s\S]*?class="h-full min-h-0"[\s\S]*?<Dashboard \/>/,
  );
});

test("message list root keeps flex children shrinkable", async () => {
  const source = await readFile(messageListPath, "utf8");

  assert.match(
    source,
    /<div class="relative flex min-h-0 flex-1 flex-col">/,
  );
});

test("message viewport wrapper can shrink before overflow kicks in", async () => {
  const source = await readFile(messageListPath, "utf8");

  assert.match(
    source,
    /<div class="relative flex-1 min-h-0 overflow-hidden">/,
  );
  assert.match(
    source,
    /<div\s+class="mx-auto mt-auto flex w-full max-w-5xl flex-col gap-3 px-3 py-3 sm:px-6 sm:py-4"/,
  );
  assert.doesNotMatch(source, /min-h-full/);
});

test("dashboard message panel allows nested scroll regions to shrink", async () => {
  const source = await readFile(dashboardPagePath, "utf8");

  assert.match(
    source,
    /<div class="relative flex h-full w-full overflow-hidden p-2\.5 sm:p-3\.5 lg:gap-3\.5">/,
  );
  assert.match(
    source,
    /<div class="shell-card relative flex min-h-0 min-w-0 flex-1 flex-col overflow-hidden">/,
  );
  assert.match(
    source,
    /<div class="flex min-h-0 flex-1 flex-col">[\s\S]*?<MessageList \/>/,
  );
});

test("message pane declares touch scrolling behavior", async () => {
  const source = await readFile(messageListPath, "utf8");

  assert.match(source, /-webkit-overflow-scrolling: touch;/);
  assert.match(source, /touch-action: pan-y;/);
  assert.match(source, /overscroll-behavior-y: contain;/);
});

test("conversation list declares touch scrolling behavior", async () => {
  const source = await readFile(conversationListPath, "utf8");

  assert.match(source, /conversation-scroll-region/);
  assert.match(source, /-webkit-overflow-scrolling: touch;/);
  assert.match(source, /touch-action: pan-y;/);
  assert.match(source, /overscroll-behavior-y: contain;/);
});
