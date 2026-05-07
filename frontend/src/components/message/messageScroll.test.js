import test from "node:test";
import assert from "node:assert/strict";

import {
  MessageScrollAlignment,
  canApplyBottomAlignment,
  getBottomAlignmentRequest,
  scrollContainerToBottom,
} from "./messageScroll.js";

test("conversation changes align the message list instantly", () => {
  assert.deepEqual(
    getBottomAlignmentRequest(MessageScrollAlignment.ConversationChange),
    { behavior: "auto", delayMs: 0 },
  );
});

test("outgoing messages keep smooth follow-up scrolling", () => {
  assert.deepEqual(
    getBottomAlignmentRequest(MessageScrollAlignment.OutgoingMessage),
    { behavior: "smooth", delayMs: 300 },
  );
});

test("conversation changes defer bottom alignment until loading finishes", () => {
  assert.equal(
    canApplyBottomAlignment({
      pendingBottomAlignment: { behavior: "auto", delayMs: 0 },
      loading: true,
      showLoading: false,
      messageContainer: {},
    }),
    false,
  );
});

test("bottom alignment runs only after the loader is hidden and a container exists", () => {
  assert.equal(
    canApplyBottomAlignment({
      pendingBottomAlignment: { behavior: "auto", delayMs: 0 },
      loading: false,
      showLoading: false,
      messageContainer: {},
    }),
    true,
  );
});

test("scrollContainerToBottom uses the container height as the target", () => {
  let scrollRequest = null;
  const container = {
    scrollHeight: 3200,
    scrollTo(options) {
      scrollRequest = options;
    },
  };

  scrollContainerToBottom(container, "smooth");

  assert.deepEqual(scrollRequest, {
    top: 3200,
    behavior: "smooth",
  });
});

test("scrollContainerToBottom ignores missing containers", () => {
  assert.doesNotThrow(() => {
    scrollContainerToBottom(null, "auto");
  });
});
