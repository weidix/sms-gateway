<script>
  import { onMount } from 'svelte';
  import { api, ApiError, openConversationStream } from '$lib/api.js';
  import Button from '$lib/components/Button.svelte';
  import Dialog from '$lib/components/Dialog.svelte';
  import { toastError, toastOk, toastWarn } from '$lib/stores/toast.svelte.js';
  import { simDisplayName } from '$lib/format.js';
  import ConversationList from '$lib/workspace/ConversationList.svelte';
  import ThreadPane from '$lib/workspace/ThreadPane.svelte';
  import Composer from '$lib/workspace/Composer.svelte';
  import NewMessageDialog from '$lib/workspace/NewMessageDialog.svelte';
  import {
    PAGE_SIZE,
    createOutgoingMessage,
    mergeConversations,
    mergeNewestMessages,
    mergeOlderMessages,
    normalizePage,
    resolveSimId,
    uniqueContactId
  } from '$lib/workspace/messages.js';

  let sims = $state([]);
  let simsState = $state('loading');

  let conversations = $state([]);
  let listState = $state('loading');
  let listError = $state('');

  let selectedId = $state(null);
  let messages = $state([]);
  let total = $state(0);
  let loadedPages = $state(0);
  let threadState = $state('idle');
  let threadError = $state('');
  let loadingEarlier = $state(false);
  let refreshing = $state(false);

  let composerSimId = $state('');
  let draft = $state('');
  let sending = $state(false);

  let newOpen = $state(false);
  let newSending = $state(false);
  let newError = $state('');

  let deleteOpen = $state(false);
  let deleting = $state(false);

  let listRequest = 0;
  let threadRequest = 0;
  let threadAbort = null;

  const selectedConversation = $derived(
    conversations.find((conversation) => conversation.contact?.id === selectedId) ?? null
  );

  const activeLineId = $derived.by(() => {
    if (selectedConversation?.sms_preview?.sim_id) return selectedConversation.sms_preview.sim_id;
    if (composerSimId) return composerSimId;
    return sims[0]?.id ?? '';
  });

  /** @param {unknown} error */
  function errorText(error) {
    if (error instanceof ApiError) return error.message;
    if (error instanceof Error && error.message) return error.message;
    return 'Something went wrong.';
  }

  async function loadSims() {
    simsState = 'loading';
    try {
      const payload = await api.simCards();
      sims = Array.isArray(payload) ? payload : [];
      simsState = 'ready';
      if (!composerSimId) composerSimId = resolveSimId(sims, '');
    } catch {
      simsState = 'error';
    }
  }

  /** @param {{ silent?: boolean }} [options] */
  async function loadConversations({ silent = false } = {}) {
    const request = ++listRequest;
    if (!silent) listState = 'loading';
    listError = '';
    try {
      const payload = await api.conversations();
      if (request !== listRequest) return;
      conversations = Array.isArray(payload) ? payload : [];
      listState = 'ready';
      if (selectedId && !conversations.some((item) => item.contact?.id === selectedId)) {
        closeThread();
      }
    } catch (error) {
      if (request !== listRequest) return;
      if (silent) return;
      listState = 'error';
      listError = errorText(error);
    }
  }

  /** @param {{ keepDraft?: boolean }} [options] */
  function closeThread({ keepDraft = false } = {}) {
    threadAbort?.abort();
    threadAbort = null;
    threadRequest += 1;
    selectedId = null;
    messages = [];
    total = 0;
    loadedPages = 0;
    threadState = 'idle';
    threadError = '';
    loadingEarlier = false;
    if (!keepDraft) draft = '';
  }

  function openNewMessage() {
    newError = '';
    newOpen = true;
  }

  /** @param {string} contactId */
  async function loadThread(contactId) {
    threadAbort?.abort();
    const controller = new AbortController();
    threadAbort = controller;
    const request = ++threadRequest;
    threadState = 'loading';
    threadError = '';
    try {
      const payload = await api.smsPage({
        page: 1,
        perPage: PAGE_SIZE,
        contactId,
        signal: controller.signal
      });
      if (request !== threadRequest || selectedId !== contactId) return;
      messages = normalizePage(payload);
      total = Number(payload?.total ?? messages.length);
      loadedPages = 1;
      threadState = 'ready';
      const entry = conversations.find((item) => item.contact?.id === contactId);
      if (entry && Number(entry.sms_preview?.status) === 0) {
        loadConversations({ silent: true });
      }
    } catch (error) {
      if (controller.signal.aborted || request !== threadRequest) return;
      threadState = 'error';
      threadError = errorText(error);
    } finally {
      if (threadAbort === controller) threadAbort = null;
    }
  }

  /** @param {Record<string, any>} conversation */
  async function openConversation(conversation) {
    const contactId = conversation?.contact?.id;
    if (!contactId) return;
    selectedId = contactId;
    messages = [];
    total = 0;
    loadedPages = 0;
    loadingEarlier = false;
    draft = '';
    composerSimId = resolveSimId(sims, conversation?.sms_preview?.sim_id);
    await loadThread(contactId);
  }

  /** @param {Record<string, any>} conversation */
  function handleSelect(conversation) {
    const contactId = conversation?.contact?.id;
    if (!contactId || contactId === selectedId) return;
    openConversation(conversation);
  }

  async function loadEarlier() {
    const conversation = selectedConversation;
    if (!conversation || loadingEarlier || threadState !== 'ready') return;
    if (messages.length >= total) return;
    const contactId = conversation.contact.id;
    const nextPage = loadedPages + 1;
    loadingEarlier = true;
    try {
      const payload = await api.smsPage({ page: nextPage, perPage: PAGE_SIZE, contactId });
      if (selectedId !== contactId) return;
      messages = mergeOlderMessages(messages, normalizePage(payload));
      total = Number(payload?.total ?? total);
      loadedPages = nextPage;
    } catch (error) {
      toastError(`Could not load earlier messages. ${errorText(error)}`);
    } finally {
      loadingEarlier = false;
    }
  }

  async function refreshThreadNewest() {
    const contactId = selectedId;
    if (!contactId) return;
    try {
      const payload = await api.smsPage({ page: 1, perPage: PAGE_SIZE, contactId });
      if (selectedId !== contactId) return;
      messages = mergeNewestMessages(messages, normalizePage(payload));
      total = Number(payload?.total ?? total);
      conversations = conversations.map((item) =>
        item.contact?.id === contactId
          ? { ...item, sms_preview: { ...item.sms_preview, status: 1 } }
          : item
      );
    } catch {
      // Live refresh is best effort; the visible thread stays intact.
    }
  }

  async function refreshInbox() {
    const lineId = activeLineId;
    if (!lineId) {
      toastWarn('No line selected to refresh.');
      return;
    }
    if (refreshing) return;
    refreshing = true;
    try {
      await api.refreshSim(lineId);
      await Promise.all([
        loadConversations({ silent: true }),
        selectedId ? refreshThreadNewest() : Promise.resolve()
      ]);
      const label = simDisplayName(sims.find((sim) => sim.id === lineId) ?? null, lineId);
      toastOk(`Inbox refreshed for ${label}.`);
    } catch (error) {
      toastError(`Inbox refresh failed. ${errorText(error)}`);
    } finally {
      refreshing = false;
    }
  }

  async function sendMessage() {
    const conversation = selectedConversation;
    if (!conversation || sending) return;
    const text = draft.trim();
    if (!text) return;
    const simId = composerSimId || resolveSimId(sims, conversation.sms_preview?.sim_id);
    if (!simId) {
      toastWarn('No line is available to send this message.');
      return;
    }

    const contact = { id: conversation.contact.id, name: conversation.contact.name };
    const optimistic = createOutgoingMessage({
      contactId: contact.id,
      simId,
      message: text
    });

    sending = true;
    draft = '';
    messages = [...messages, optimistic];

    try {
      const result = await api.sendSms({ simId, contact, message: text, isNew: false });
      const serverId = result?.sms_id ?? optimistic.id;
      messages = messages.map((item) =>
        item.id === optimistic.id ? { ...item, id: serverId, status: 1 } : item
      );
      await loadConversations({ silent: true });
      toastOk('Message sent.');
    } catch (error) {
      messages = messages.map((item) =>
        item.id === optimistic.id ? { ...item, status: 3 } : item
      );
      toastError(`Message could not be sent. ${errorText(error)}`);
    } finally {
      sending = false;
    }
  }

  /** @param {{ contact: string, simId: string, message: string }} payload */
  async function createMessage({ contact, simId, message }) {
    if (newSending) return;
    newSending = true;
    newError = '';
    const draftContact = { id: uniqueContactId(), name: contact };
    try {
      const result = await api.sendSms({
        simId,
        contact: draftContact,
        message,
        isNew: true
      });
      const contactId = result?.contact_id ?? draftContact.id;
      newOpen = false;
      await loadConversations({ silent: true });
      const created = conversations.find((item) => item.contact?.id === contactId);
      if (created) {
        await openConversation(created);
      } else {
        await openConversation({
          contact: { id: contactId, name: contact },
          sms_preview: { sim_id: simId }
        });
      }
      toastOk(`Conversation started with ${contact}.`);
    } catch (error) {
      newError = errorText(error);
      toastError(`Could not start the conversation. ${newError}`);
    } finally {
      newSending = false;
    }
  }

  async function deleteConversation() {
    const conversation = selectedConversation;
    if (!conversation || deleting) return;
    const name = conversation.contact?.name || 'this contact';
    deleting = true;
    try {
      await api.deleteConversation(conversation.contact.id);
      deleteOpen = false;
      closeThread();
      await loadConversations({ silent: true });
      toastOk(`Conversation with ${name} deleted.`);
    } catch (error) {
      toastError(`Could not delete the conversation. ${errorText(error)}`);
    } finally {
      deleting = false;
    }
  }

  /** @param {unknown[]} next Partial SSE update, or a full history refresh. */
  function applyConversations(next) {
    if (!Array.isArray(next)) return;
    const previous = selectedConversation;
    const previousPreview = previous?.sms_preview?.timestamp ?? null;
    // The SSE payload contains only the conversations that just archived an
    // SMS. Replacing the sidebar with it dropped every other contact.
    conversations = mergeConversations(conversations, next);
    listState = 'ready';
    listError = '';
    if (selectedId && !conversations.some((item) => item.contact?.id === selectedId)) {
      closeThread();
      return;
    }
    const updated = conversations.find((item) => item.contact?.id === selectedId);
    const nextPreview = updated?.sms_preview?.timestamp ?? null;
    if (
      selectedId &&
      !sending &&
      threadState === 'ready' &&
      nextPreview &&
      nextPreview !== previousPreview
    ) {
      refreshThreadNewest();
    }
  }

  onMount(() => {
    loadSims();
    loadConversations();

    const unsubscribe = openConversationStream({ onConversations: applyConversations });

    return () => unsubscribe();
  });
</script>

<svelte:head>
  <title>Workspace — SMS Gateway</title>
</svelte:head>

{#snippet threadFooter()}
  <Composer
    {sims}
    {simsState}
    bind:simId={composerSimId}
    bind:value={draft}
    {sending}
    onsend={sendMessage}
    onretry={loadSims}
  />
{/snippet}

<div class="page page--fill workspace-page">
  <div class="workspace" class:workspace--thread-open={Boolean(selectedId)}>
    <div class="pane pane--list">
      <ConversationList
        {conversations}
        status={listState}
        error={listError}
        {selectedId}
        onselect={handleSelect}
        onnew={openNewMessage}
        onretry={() => loadConversations()}
      />
    </div>

    <div class="pane pane--thread">
      <ThreadPane
        conversation={selectedConversation}
        {messages}
        {total}
        status={threadState}
        error={threadError}
        {loadingEarlier}
        {refreshing}
        {sims}
        onnew={openNewMessage}
        onretry={() => selectedId && loadThread(selectedId)}
        onloadearlier={loadEarlier}
        onrefresh={refreshInbox}
        ondelete={() => (deleteOpen = true)}
        onback={() => closeThread({ keepDraft: true })}
        footer={selectedConversation ? threadFooter : null}
      />
    </div>
  </div>
</div>

<NewMessageDialog
  bind:open={newOpen}
  {sims}
  {simsState}
  defaultSimId={composerSimId || activeLineId}
  busy={newSending}
  error={newError}
  onsubmit={createMessage}
/>

<Dialog bind:open={deleteOpen} title="Delete conversation?" size="sm">
  <p class="confirm">This removes the message history and cannot be undone.</p>

  {#snippet footer()}
    <Button variant="ghost" onclick={() => (deleteOpen = false)} disabled={deleting}>Cancel</Button>
    <Button variant="danger" onclick={deleteConversation} loading={deleting}>Delete</Button>
  {/snippet}
</Dialog>

<style>
  .page.workspace-page {
    padding: 0;
    gap: 0;
  }

  .workspace {
    display: grid;
    grid-template-columns: 310px minmax(0, 1fr);
    gap: 16px;
    flex: 1;
    min-width: 0;
    min-height: 0;
    padding: 22px 28px 26px;
  }

  .pane {
    display: flex;
    min-width: 0;
    min-height: 0;
  }

  .confirm {
    margin: 0;
    color: var(--ink-2);
    font-size: var(--text-sm);
    line-height: 1.55;
  }

  @container page (max-width: 899px) {
    .workspace {
      grid-template-columns: minmax(0, 1fr);
      padding: 12px 16px 18px;
    }

    .pane--list {
      animation: pane-in-left 190ms var(--ease-out) both;
    }

    .pane--thread {
      display: none;
    }

    .workspace--thread-open .pane--thread {
      display: flex;
      animation: pane-in-right 200ms var(--ease-out) both;
    }

    .workspace--thread-open .pane--list {
      display: none;
    }
  }

  @keyframes pane-in-right {
    from {
      opacity: 0;
      transform: translateX(18px);
    }
  }

  @keyframes pane-in-left {
    from {
      opacity: 0;
      transform: translateX(-14px);
    }
  }
</style>
