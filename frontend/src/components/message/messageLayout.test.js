import test from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";

const appPath = new URL("../../App.svelte", import.meta.url);
const appCssPath = new URL("../../app.css", import.meta.url);
const messageListPath = new URL("./MessageList.svelte", import.meta.url);
const messageHeaderPath = new URL("./MessageHeader.svelte", import.meta.url);
const messageInputPath = new URL("./MessageInputOptimized.svelte", import.meta.url);
const simSelectorPath = new URL("./SimSelector.svelte", import.meta.url);
const dashboardPagePath = new URL("../../pages/Dashboard.svelte", import.meta.url);
const sidebarPath = new URL("../layout/Sidebar.svelte", import.meta.url);
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

test("composer keeps the sim picker overlay from shrinking the input row", async () => {
  const messageInputSource = await readFile(messageInputPath, "utf8");
  const simSelectorSource = await readFile(simSelectorPath, "utf8");

  assert.match(
    messageInputSource,
    /<div class="flex flex-col gap-3 md:flex-row xl:w-auto xl:shrink-0">/,
  );
  assert.match(
    simSelectorSource,
    /--sim-trigger-width:\s*220px;/,
  );
  assert.match(
    simSelectorSource,
    /--sim-panel-width:\s*320px;/,
  );
});

test("composer input controls share the same pill radius", async () => {
  const messageInputSource = await readFile(messageInputPath, "utf8");
  const simSelectorSource = await readFile(simSelectorPath, "utf8");

  assert.match(
    messageInputSource,
    /class="shell-input h-12 rounded-\[22px\] pl-11 pr-4 sm:h-14 sm:rounded-\[24px\] sm:pl-12"/,
  );
  assert.match(
    messageInputSource,
    /class=\{`shell-button h-12 min-w-\[112px\] shrink-0 rounded-\[22px\] sm:h-14 sm:min-w-\[124px\] sm:rounded-\[24px\]/,
  );
  assert.match(
    simSelectorSource,
    /rounded-\[24px\]/,
  );
});

test("primary action buttons keep readable contrast on hover and press", async () => {
  const source = await readFile(appCssPath, "utf8");

  assert.match(
    source,
    /\.shell-button-primary:hover\s*\{[\s\S]*background:\s*linear-gradient\(/,
  );
  assert.match(
    source,
    /\.shell-button-primary:hover\s*\{[\s\S]*color:\s*#fbf4ea;/,
  );
  assert.match(
    source,
    /\.shell-button-primary:active\s*\{[\s\S]*background:\s*linear-gradient\(/,
  );
  assert.match(
    source,
    /\.shell-button-primary:active\s*\{[\s\S]*color:\s*#fbf4ea;/,
  );
  assert.match(
    source,
    /html\s*\{[\s\S]*font-size:\s*90%;/,
  );
  assert.match(
    source,
    /body\s*\{[\s\S]*min-height:\s*100dvh;/,
  );
  assert.match(
    source,
    /#app\s*\{[\s\S]*min-height:\s*100dvh;[\s\S]*height:\s*100dvh;/,
  );
  assert.match(
    source,
    /@media \(max-width: 767px\)\s*\{[\s\S]*input:not\(\[type="checkbox"\]\):not\(\[type="radio"\]\),[\s\S]*textarea,[\s\S]*select\s*\{[\s\S]*font-size:\s*16px;/,
  );
});

test("sidebar preserves search focus glow without losing scroll clipping", async () => {
  const sidebarSource = await readFile(sidebarPath, "utf8");
  const conversationListSource = await readFile(conversationListPath, "utf8");

  assert.match(
    sidebarSource,
    /<div class="shell-card flex h-full w-full flex-col overflow-visible p-3 sm:p-4">/,
  );
  assert.match(
    sidebarSource,
    /<div class="min-h-0 flex-1 overflow-visible">/,
  );
  assert.match(
    conversationListSource,
    /<div class="min-h-0 flex-1 overflow-hidden">/,
  );
});

test("sim selector popup anchors to the trigger on mobile without drifting right", async () => {
  const simSelectorSource = await readFile(simSelectorPath, "utf8");

  assert.match(
    simSelectorSource,
    /class="absolute bottom-\[calc\(100%\+0\.75rem\)\] left-0 z-20 w-full max-w-\[calc\(100vw-1\.5rem\)\] overflow-y-hide scrollbar-hide md:w-\[var\(--sim-panel-width\)\] md:max-w-none"/,
  );
});

test("message workspace titles and actions stay compact through tablet widths", async () => {
  const dashboardSource = await readFile(dashboardPagePath, "utf8");
  const messageHeaderSource = await readFile(messageHeaderPath, "utf8");
  const messageInputSource = await readFile(messageInputPath, "utf8");
  const messageListSource = await readFile(messageListPath, "utf8");

  assert.match(
    dashboardSource,
    /class="shell-button h-10 min-w-0 max-w-\[calc\(100%-3\.5rem\)\] px-3 py-2"/,
  );
  assert.match(
    dashboardSource,
    /<span class="truncate text-xs font-medium sm:text-sm">/,
  );
  assert.match(
    messageHeaderSource,
    /const recipientTitleClass =\s*"mt-0\.5 block truncate text-\[0\.95rem\] font-medium sm:text-base lg:text-\[1\.05rem\]";/,
  );
  assert.match(
    messageInputSource,
    /<div class="flex flex-col gap-3 md:flex-row xl:w-auto xl:shrink-0">/,
  );
  assert.match(
    messageInputSource,
    /class="shell-input h-12 rounded-\[22px\] pl-11 pr-4 sm:h-14 sm:rounded-\[24px\] sm:pl-12"/,
  );
  assert.match(
    messageInputSource,
    /class=\{`shell-button h-12 min-w-\[112px\] shrink-0 rounded-\[22px\] sm:h-14 sm:min-w-\[124px\] sm:rounded-\[24px\]/,
  );
  assert.match(
    messageListSource,
    /<div class="grid grid-cols-1 gap-3 md:grid-cols-2">/,
  );
});
