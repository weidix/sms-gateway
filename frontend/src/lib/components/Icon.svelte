<script>
  /**
   * Inline icon set. Single 24px stroke grid so every glyph shares weight,
   * caps and optical size. Unknown names render a small centred square.
   */
  let { name, size = 16, stroke = 1.6, label = '' } = $props();

  const icons = {
    message:
      '<path d="M4.5 5.5h15v10h-9l-4.5 3.5v-3.5h-1.5z"/>',
    radio:
      '<circle cx="12" cy="12" r="1.6" fill="currentColor" stroke="none"/><path d="M8.8 8.8a4.5 4.5 0 0 0 0 6.4M15.2 8.8a4.5 4.5 0 0 1 0 6.4"/><path d="M6 5.8a8.6 8.6 0 0 0 0 12.4M18 5.8a8.6 8.6 0 0 1 0 12.4"/>',
    refresh:
      '<path d="M19 12a7 7 0 1 1-2.1-5"/><path d="M19 4.5V9h-4.5"/>',
    send:
      '<path d="M20 4 4.5 10.5l6 2.5 2.5 6z"/><path d="M20 4 10.5 13"/>',
    search:
      '<circle cx="10.5" cy="10.5" r="5.5"/><path d="m15 15 4.5 4.5"/>',
    plus: '<path d="M12 5v14M5 12h14"/>',
    minus: '<path d="M5 12h14"/>',
    'chevron-down': '<path d="m6 9.5 6 6 6-6"/>',
    'chevron-left': '<path d="m14.5 6-6 6 6 6"/>',
    'chevron-right': '<path d="m9.5 6 6 6-6 6"/>',
    'arrow-left': '<path d="M19 12H5M11 6l-6 6 6 6"/>',
    'arrow-right': '<path d="M5 12h14M13 6l6 6-6 6"/>',
    close: '<path d="M6 6l12 12M18 6 6 18"/>',
    check: '<path d="m5 12.5 4.5 4.5L19 6.5"/>',
    trash:
      '<path d="M4.5 7h15M9.5 7V4.5h5V7M6.5 7l1 12.5h9L17.5 7"/><path d="M10.5 10.5v6M13.5 10.5v6"/>',
    pen: '<path d="m4.5 19.5 1-4 10-10 3 3-10 10z"/><path d="m13.5 7.5 3 3"/>',
    phone:
      '<path d="M7 4.5h3l1.5 4-2 1.5a11 11 0 0 0 4.5 4.5l1.5-2 4 1.5v3c0 1-1 2-2 2A15.5 15.5 0 0 1 5 6.5c0-1 1-2 2-2z"/>',
    sim:
      '<path d="M6.5 4.5h7l4 4v11h-11z"/><rect x="10" y="9.5" width="4" height="4" rx="0.5"/><path d="M10 13.5v1.5M14 13.5v1.5"/>',
    terminal:
      '<rect x="3.5" y="4.5" width="17" height="15" rx="1.5"/><path d="m7.5 9 3 3-3 3M13 15.5h4"/>',
    inbox:
      '<path d="M4.5 5.5h15v13h-15z"/><path d="M4.5 12h5l1.5 2h2l1.5-2h5"/>',
    copy:
      '<rect x="8.5" y="8.5" width="10" height="10" rx="1"/><path d="M15.5 5.5h-11v11"/>',
    logout:
      '<path d="M14.5 5.5h4v13h-4"/><path d="M4.5 12h10M11 8.5l3.5 3.5-3.5 3.5"/>',
    alert:
      '<path d="M12 4.5 21 19.5H3z"/><path d="M12 10v4.5M12 17.2v.3"/>',
    info:
      '<circle cx="12" cy="12" r="8"/><path d="M12 11v5M12 8.2v.3"/>',
    clock:
      '<circle cx="12" cy="12" r="8"/><path d="M12 7.5V12l3 2"/>',
    zap: '<path d="M13 3.5 5.5 13.5H11l-1 7 8-10.5H12.5z"/>',
    play: '<path d="M8 5.5v13l10.5-6.5z"/>',
    more:
      '<circle cx="6" cy="12" r="1.1" fill="currentColor" stroke="none"/><circle cx="12" cy="12" r="1.1" fill="currentColor" stroke="none"/><circle cx="18" cy="12" r="1.1" fill="currentColor" stroke="none"/>',
    lock:
      '<rect x="6" y="10.5" width="12" height="9" rx="1"/><path d="M8.5 10.5V8a3.5 3.5 0 0 1 7 0v2.5M12 14v2.5"/>',
    user:
      '<circle cx="12" cy="8.5" r="3.5"/><path d="M5 20c.5-4 3.5-6 7-6s6.5 2 7 6"/>',
    sliders:
      '<path d="M4.5 8h15M4.5 16h15"/><circle cx="9" cy="8" r="2"/><circle cx="15" cy="16" r="2"/>',
    storage:
      '<ellipse cx="12" cy="6.5" rx="7" ry="2.5"/><path d="M5 6.5v11c0 1.4 3.1 2.5 7 2.5s7-1.1 7-2.5v-11"/><path d="M5 12c0 1.4 3.1 2.5 7 2.5s7-1.1 7-2.5"/>',
    dot: '<circle cx="12" cy="12" r="2.5" fill="currentColor" stroke="none"/>',
    filter: '<path d="M4.5 6h15l-6 7v6l-3-2v-4z"/>',
    external: '<path d="M13.5 5.5h5v5M18.5 5.5 11 13M9 6H5.5v13h13V15"/>',
    temperature:
      '<path d="M10 14.5V6.5a2 2 0 0 1 4 0v8a4 4 0 1 1-4 0z"/><path d="M12 10.5v4.5"/>',
    key:
      '<circle cx="8.5" cy="8.5" r="3.5"/><path d="m11 11 8 8M16 16l2-2M18.5 18.5l2-2"/>'
  };
</script>

<svg
  class="icon"
  width={size}
  height={size}
  viewBox="0 0 24 24"
  fill="none"
  stroke="currentColor"
  stroke-width={stroke}
  stroke-linecap="round"
  stroke-linejoin="round"
  role={label ? 'img' : undefined}
  aria-hidden={label ? undefined : 'true'}
  aria-label={label || undefined}
  focusable="false"
>
  {@html icons[name] ?? icons.dot}
</svg>

<style>
  .icon {
    flex: none;
    display: block;
  }
</style>
