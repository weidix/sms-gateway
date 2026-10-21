<script>
  import '../app.css';
  import { onNavigate } from '$app/navigation';
  import Toaster from '$lib/components/Toaster.svelte';

  let { children } = $props();

  /** @param {string} pathname */
  function routeIndex(pathname) {
    if (pathname.startsWith('/fleet')) return 2;
    if (pathname.startsWith('/login')) return 0;
    return 1;
  }

  onNavigate((navigation) => {
    if (typeof document === 'undefined' || typeof document.startViewTransition !== 'function') {
      return undefined;
    }
    if (window.matchMedia('(prefers-reduced-motion: reduce)').matches) return undefined;

    const from = navigation.from?.url?.pathname ?? '/';
    const to = navigation.to?.url?.pathname ?? '/';
    document.documentElement.dataset.nav =
      routeIndex(to) >= routeIndex(from) ? 'forward' : 'back';

    return new Promise((resolve) => {
      document.startViewTransition(async () => {
        resolve();
        await navigation.complete;
      });
    });
  });
</script>

{@render children()}
<Toaster />
