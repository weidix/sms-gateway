export const MessageScrollAlignment = Object.freeze({
  ConversationChange: "conversation-change",
  IncomingUpdate: "incoming-update",
  OutgoingMessage: "outgoing-message",
});

export function getBottomAlignmentRequest(source) {
  switch (source) {
    case MessageScrollAlignment.OutgoingMessage:
      return { behavior: "smooth", delayMs: 300 };
    case MessageScrollAlignment.IncomingUpdate:
    case MessageScrollAlignment.ConversationChange:
    default:
      return { behavior: "auto", delayMs: 0 };
  }
}

export function canApplyBottomAlignment({
  pendingBottomAlignment,
  loading,
  showLoading,
  messageContainer,
}) {
  return Boolean(
    pendingBottomAlignment &&
      !loading &&
      !showLoading &&
      messageContainer,
  );
}

export function scrollContainerToBottom(container, behavior = "auto") {
  if (!container) {
    return;
  }

  container.scrollTo({
    top: container.scrollHeight,
    behavior,
  });
}
