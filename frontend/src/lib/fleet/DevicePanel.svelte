<script>
  import Panel from '$lib/components/Panel.svelte';
  import Facts from './Facts.svelte';

  let { sim, card = null } = $props();

  const alias = $derived(card?.alias?.trim() ?? '');
  const phone = $derived(card?.phone_number?.trim() ?? '');

  const facts = $derived([
    ...(alias && phone ? [{ label: 'Phone', value: phone }] : []),
    { label: 'Operator', value: sim?.operator_info?.operator_name ?? '' },
    { label: 'Model', value: sim?.model_info?.model ?? '' },
    { label: 'IMSI', value: card?.imsi ?? '' },
    { label: 'COM port', value: sim?.com_port ?? '' },
    { label: 'Baud', value: sim?.baud_rate ? `${sim.baud_rate}` : '' }
  ]);
</script>

<Panel title="Device">
  <Facts items={facts} />
</Panel>
