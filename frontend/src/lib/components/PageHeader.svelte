<script>
  let { eyebrow = '', title = '', meta, actions } = $props();
</script>

<header class="page-header">
  <div class="page-header__text">
    {#if eyebrow}<span class="eyebrow">{eyebrow}</span>{/if}
    <h1 class="page-title">{title}</h1>
    {#if meta}<div class="page-header__meta">{@render meta()}</div>{/if}
  </div>
  {#if actions}<div class="page-header__actions">{@render actions()}</div>{/if}
</header>
