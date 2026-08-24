<script>
  import Icon from "@iconify/svelte";
  import { SmsStatus } from "../../stores/conversation";
  import { quintOut } from "svelte/easing";

  let { message, isNewMessage = false } = $props();

  const urlRegex =
    /https?:\/\/[a-zA-Z0-9][-a-zA-Z0-9@:%._\+~#=]{1,256}\.[a-zA-Z0-9()]{1,6}\b(?:[-a-zA-Z0-9()@:%_\+.~#?&//=]*)/g;

  const verificationCodeRegex =
    /(?:验证码|校验码|(?<!号)码|code|Code)[^0-9]*([0-9]{4,8})/gi;

  function formatMessage(text) {
    if (!text) return "";

    let formattedText = text;

    const urlMatches = text.match(urlRegex);
    if (urlMatches) {
      urlMatches.forEach((url) => {
        const cleanUrl = url.replace(/[（）]/g, "");
        formattedText = formattedText.replace(
          url,
          `<a href="${cleanUrl}" target="_blank" rel="noopener noreferrer" class="font-medium underline underline-offset-4 transition-opacity duration-150 hover:opacity-75" style="color: inherit; text-decoration-color: var(--line-strong);">${cleanUrl}</a>`,
        );
      });
    }

    const codeMatches = text.matchAll(verificationCodeRegex);
    for (const match of codeMatches) {
      const [fullMatch, code] = match;
      const prefix = fullMatch.slice(0, fullMatch.lastIndexOf(code));
      formattedText = formattedText.replace(
        fullMatch,
        `${prefix}<span class="shell-mono mx-0.5 inline-flex cursor-pointer items-center justify-center gap-1 rounded-sm border px-1.5 py-0.5 font-semibold transition-colors duration-150 hover:opacity-80" style="border-color: currentColor; color: inherit;" onclick="(function(event){
            event.preventDefault();
            event.stopPropagation();
            navigator.clipboard.writeText('${code}');
            const toast = document.createElement('div');
            toast.className = 'fixed bottom-24 left-1/2 z-[80] -translate-x-1/2 rounded-md px-3 py-1.5 text-xs font-medium opacity-0 transition-opacity duration-200';
            toast.style.background = 'var(--panel-contrast)';
            toast.style.color = 'var(--paper-strong)';
            toast.style.boxShadow = 'var(--shadow-strong)';
            toast.textContent = 'Code copied';
            document.body.appendChild(toast);
            requestAnimationFrame(() => toast.style.opacity = '1');
            setTimeout(() => {
              toast.style.opacity = '0';
              setTimeout(() => toast.remove(), 200);
            }, 1600);          })(event)" title="点击复制验证码">${code}<svg class="w-3 h-3" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path fill="currentColor" d="M16 1H4c-1.1 0-2 .9-2 2v14h2V3h12V1zm3 4H8c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h11c1.1 0 2-.9 2-2V7c0-1.1-.9-2-2-2zm0 16H8V7h11v14z"/></svg></span>`,
      );
    }

    return formattedText;
  }

  function slideDown(node, { duration = 240, easing = quintOut }) {
    if (!isNewMessage) return {};

    const height = node.offsetHeight;

    return {
      duration,
      css: (t) => {
        const eased = easing(t);
        return `
          overflow: hidden;
          height: ${eased * height}px;
          opacity: ${t < 0.5 ? t * 2 : 1};
          transform: translateY(${(1 - eased) * 6}px);
        `;
      },
    };
  }
</script>

<div
  class="message-wrapper mb-2 flex flex-row"
  class:justify-end={message.send}
  class:justify-start={!message.send}
  in:slideDown={{ duration: 240 }}
>
  <div
    class="relative max-w-[82%] sm:max-w-[72%] lg:max-w-[60%] xl:max-w-[55%]"
  >
    {#if message.send && message.status !== undefined}
      <div class="absolute top-1/2 -left-5 -translate-y-1/2 transform">
        {#if message.status === SmsStatus.Loading}
          <div
            class="h-2.5 w-2.5 animate-spin rounded-full border"
            style="border-color: var(--line-strong); border-top-color: var(--text-muted);"
          ></div>
        {:else if message.status === SmsStatus.Failed}
          <Icon
            icon="mage:information-circle-fill"
            class="h-3.5 w-3.5"
            style="color: var(--danger-strong);"
          />
        {:else if message.status === SmsStatus.Read}
          <Icon
            icon="carbon:checkmark"
            class="h-3.5 w-3.5"
            style="color: var(--success-strong);"
          />
        {/if}
      </div>
    {/if}

    <div
      class={`relative rounded-lg px-3.5 py-2.5 text-sm ${
        message.send
          ? 'bg-[var(--panel-contrast)] text-[var(--paper-strong)]'
          : 'border bg-[var(--panel)] text-[var(--text-strong)]'
      }`}
      style={!message.send ? 'border-color: var(--line-soft);' : ''}
    >
      <p class="overflow-hidden whitespace-pre-wrap break-words leading-6">
        {@html formatMessage(message.message)}
      </p>
    </div>
  </div>
</div>

<style>
  .message-wrapper {
    will-change: height, transform, opacity;
  }

</style>
