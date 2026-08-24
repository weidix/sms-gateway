import test from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";

const commonModalPath = new URL("../common/Modal.svelte", import.meta.url);
const simCardModalPath = new URL("./SimCardModal.svelte", import.meta.url);
const simCardTabContentPath = new URL("./SimCardTabContent.svelte", import.meta.url);
const simCardTechnicalInfoPath = new URL("./SimCardTechnicalInfo.svelte", import.meta.url);
const signalStrengthIndicatorPath = new URL("./SignalStrengthIndicator.svelte", import.meta.url);
const atDebugModalPath = new URL("./AtDebugModal.svelte", import.meta.url);

test("sim card modal titles scale down before desktop", async () => {
  const modalSource = await readFile(simCardModalPath, "utf8");
  const tabContentSource = await readFile(simCardTabContentPath, "utf8");
  const technicalInfoSource = await readFile(simCardTechnicalInfoPath, "utf8");
  const atDebugModalSource = await readFile(atDebugModalPath, "utf8");

  assert.match(
    modalSource,
    /class="shell-heading text-base font-semibold leading-tight"/,
  );
  assert.match(
    tabContentSource,
    /class="shell-heading shell-mono mt-1 text-base font-semibold"/,
  );
  assert.match(
    technicalInfoSource,
    /class="shell-heading mt-1 text-base font-semibold"/,
  );
  assert.match(
    atDebugModalSource,
    /class="shell-heading text-base font-semibold leading-tight"/,
  );
});

test("signal strength icons reserve stable space in the title and align bars with the reading", async () => {
  const technicalInfoSource = await readFile(simCardTechnicalInfoPath, "utf8");
  const indicatorSource = await readFile(signalStrengthIndicatorPath, "utf8");

  assert.match(
    technicalInfoSource,
    /flex h-8 w-10 items-end justify-center rounded-md border/,
  );
  assert.match(
    indicatorSource,
    /<div class="signal-strength-compact flex h-\[0\.9rem\] items-end justify-center gap-\[0\.16rem\]">/,
  );
  assert.match(
    indicatorSource,
    /#each Array\(3\) as _, i/,
  );
  assert.match(
    indicatorSource,
    /class="signal-strength-bar w-\[0\.2rem\] shrink-0 rounded-sm"/,
  );
  assert.match(
    indicatorSource,
    /<div class="mt-1 flex items-end gap-3">/,
  );
  assert.match(
    indicatorSource,
    /class="text-sm font-medium leading-none"/,
  );
  assert.match(
    indicatorSource,
    /<div class="signal-strength-bars flex items-end gap-1">/,
  );
});

test("sim detail action rows keep controls readable on mobile", async () => {
  const tabContentSource = await readFile(simCardTabContentPath, "utf8");
  const atDebugModalSource = await readFile(atDebugModalPath, "utf8");

  assert.match(
    tabContentSource,
    /<div class="flex w-full flex-col gap-2 sm:w-auto">/,
  );
  assert.match(
    tabContentSource,
    /<div class="flex w-full items-center gap-2">/,
  );
  assert.match(
    tabContentSource,
    /class="shell-button flex-1"/,
  );
  assert.match(
    tabContentSource,
    /<div class="flex w-full justify-end">[\s\S]*?<span class="shell-chip">/,
  );
  assert.match(
    atDebugModalSource,
    /<div class="flex flex-col gap-2 sm:flex-row sm:items-center">/,
  );
  assert.match(
    atDebugModalSource,
    /class=\{`shell-button w-full px-4 sm:w-auto \$\{isRunning \|\| !command\.trim\(\) \? 'cursor-not-allowed opacity-50' : 'shell-button-primary'\}`\}/,
  );
});

test("shared modal shell preserves vertical breathing room on mobile", async () => {
  const modalSource = await readFile(commonModalPath, "utf8");

  assert.match(
    modalSource,
    /class="fixed inset-0 z-50 flex items-center justify-center overflow-y-auto bg-black\/50 px-3 py-4 sm:px-4 sm:py-6 \{overlayClass\}"/,
  );
  assert.doesNotMatch(
    modalSource,
    /backdrop-blur/,
  );
  assert.match(
    modalSource,
    /class="my-auto flex w-full \{maxWidth\} max-h-\[calc\(100dvh-2rem\)\] flex-col overflow-hidden rounded-xl border sm:max-h-\[calc\(100dvh-3rem\)\] \{className\}"/,
  );
});
