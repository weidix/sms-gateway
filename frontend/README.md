# SMS Gateway — frontend

SvelteKit 2 single-page app, statically built into `dist/` and embedded in the Rust
binary with `rust-embed`. `pnpm build` is the only integration point the server needs.

```bash
pnpm install
pnpm dev      # http://localhost:5173, proxies /api to http://localhost:8951
pnpm build    # writes dist/index.html + dist/_app/...
```

The app runs client-side only (`ssr = false`); `dist/index.html` is the SPA fallback and
the Rust fallback handler serves it for deep links.

## Layout of the source

```
src/
  app.css                     design tokens, base type, shared layout scaffolding
  routes/
    +layout.svelte            mounts global CSS + toaster
    +layout.js                ssr/prerender disabled
    +error.svelte             error surface
    login/+page.svelte        sign-in screen
    (app)/+layout.svelte      top navigation shell + session bootstrap
    (app)/+page.svelte        conversation workspace
    (app)/fleet/+page.svelte  line fleet overview
    (app)/fleet/[simId]/+page.svelte
                              line detail, storage, AT console
  lib/
    api.js                    HTTP client, SSE stream, auth storage
    format.js                 date/signal/storage formatting (+ parseStorageStatus)
    health.js                 health vocabulary (status tones, failure labels)
    stores/auth.svelte.js     session state + sign in/out
    stores/toast.svelte.js    toast queue (toastOk / toastWarn / toastError)
    workspace/MessageBubble.svelte
                              canvas-drawn bubble shape + tail
    components/Logo.svelte    SIM + signal app mark (top bar, login, favicon)
    components/*.svelte       shared primitives (see below)
```

## Visual language — non-negotiable

The interface is deliberately monochrome: a cool light-gray ground, white cards lifted by
soft shadows, a slim top navigation bar and calm sans-serif typography. It should feel
quiet and modern, not like hardware instrumentation. Colour is reserved for **status only**.

- **Never** use blue, indigo, violet, purple, cyan, or teal as a UI colour.
- Non-status surfaces and controls use only the paper/ink/line tokens. Do not invent
  coloured buttons, gradients, coloured cards, or coloured links.
- Status colour is allowed only on health/connection/alert indicators via
  `Badge` and severity text: `--ok`, `--warn`, `--danger`.
- Typography roles: `--font-sans` for everything (page titles, panel titles, UI text);
  `--font-mono` only for ids, timestamps, AT output and tabular data. `--font-serif` is
  aliased to the sans face for legacy rules. Titles are 600 weight.
- Sizes: body 13.5px, secondary 12.5px, micro labels 10px, page title 24px.
- Surfaces are separated with `--shadow-card` (white cards on the gray ground), not heavy
  borders. Radii: 9px controls, 13px cards/panels, 18px dialogs, 16px message
  bubbles with a small side pointer. Buttons are pills. Popovers use `--shadow-pop`.
- The app mark is `Logo` (SIM card + signal bars): use it in the top-left of the shell,
  on the login card and as the favicon. Do not replace it with letter initials.

## Layout rules

- Every row is a CSS grid or flex row with explicit alignment (`align-items`), never a
  free-floating mix of inline blocks. Elements that share a visual line must share the
  same height, baseline, or centre line even when they live in different containers.
- Composed screens are balanced: no column may carry all controls while another is empty;
  a left column is matched by an equally weighted right column. Header actions sit on the
  title/meta block's centre line, and the right edge of every toolbar aligns with the
  content edge.
- Scroll regions use `.scroll-y` / `.page--scroll` (`scrollbar-gutter: stable`), so header
  and toolbar right edges line up with the scrolled content instead of sitting on the
  scrollbar.
- Responsive breakpoints are **container queries** on the page root (`.page` already sets
  `container-type: inline-size`), not viewport media queries. Shell-only chrome may use a
  viewport query.
- Phone layout rule set: one 16px side gutter for the topbar, every page and the workspace
  cards; a 56px topbar; 36px for every control that shares a row (inputs, selects, buttons);
  paired header actions split their row into equal widths; the fleet filter tabs split the
  row into three equal tabs; the composer becomes full-width textarea above one row of
  line select + Send, both 36px; dialogs keep 16px side margins (`100vw - 32px`). No screen
  may use a different phone gutter.
- Every popup (dialog, select list, menu, toast) must be visually checked: not clipped by
  the panel edge, opens above the trigger when near the bottom, footer buttons aligned
  right, body scrolls inside instead of overflowing the dialog.
- Empty, loading, and failure states are mandatory for each data region. Never render a
  blank panel.
- Motion is limited to 130–160ms state transitions and respects `prefers-reduced-motion`.

## Restraint — remove everything that does not help the task

The interface must stay quiet. Every extra label, badge, rule or panel makes the screen
harder to read, so default to removing:

- No decorative elements: no tick rulers, signal spines, ornamental dividers, watermark
  strips or icon confetti. A border exists only to separate a region, never to decorate.
- No duplicated information. A fact appears once per screen: if the header already shows
  the line name and health, the panel below does not repeat them; if a metric card shows
  the model, the identity list omits it.
- No label stacking. A region has one heading. Do not put an eyebrow above every title,
  and do not pair an eyebrow and a title that say the same thing.
- No badges or pills for ordinary facts (a phone number, a model, a port). Status colour
  is used only for real health/connection state.
- No non-functional chrome: no dead buttons, no search box for a four-item list, no
  metric that nobody acts on, no copy that explains the obvious. Button labels are verbs,
  field labels are nouns.
- Empty/loading/error copy is one short line; guidance appears only when it changes what
  the user can do.
- Prefer one white surface with internal spacing over a stack of extra bordered panels.

## Shared primitives

Import from `$lib/components/...`. Do not restyle shared primitives from page CSS; use
their props or add a page-local wrapper.

| Component | Props (all optional unless noted) |
| --- | --- |
| `Button` | `variant`: `secondary` (default), `primary`, `ghost`, `danger`; `size`: `sm`, `md`, `lg`; `loading`, `disabled`, `block`, `type`, `href`, `onclick`, `children` |
| `IconButton` | `name` (required), `label`, `size` (30), `variant`: `ghost`, `outline`, `onclick` |
| `Icon` | `name` (required), `size`, `stroke`, `label` — 24px stroke set, see file for names |
| `Input` | `bind:value`, `type`, `placeholder`, `disabled`, `invalid`, `id`, `autocomplete`, `onkeydown` |
| `Textarea` | `bind:value`, `rows`, `placeholder`, `disabled`, `invalid`, `onkeydown` |
| `Field` | `label`, `hint`, `error`, `forId`, `children` |
| `Select` | `bind:value`, `options: {value,label,disabled}[]`, `placeholder`, `disabled`, `label`, `invalid` |
| `Dialog` | `bind:open`, `title` (required), `description`, `size`: `sm`\|`md`\|`lg`\|`xl`, `footer` snippet, `children` |
| `Panel` | `eyebrow`, `title`, `description`, `actions` snippet, `flush`, `children` |
| `PageHeader` | `eyebrow`, `title`, `meta` snippet, `actions` snippet |
| `Metric` | `label`, `value`, `unit`, `hint`, or `children` for custom value |
| `MetricStrip` | `items: {label,value,unit?,hint?}[]`, `cols` |
| `Badge` | `tone`: `neutral`\|`outline`\|`ok`\|`warn`\|`danger`\|`idle`, `dot`, `children` |
| `Logo` | `size`, `showWordmark` (default true), `label` |
| `SignalBars` | `rssi` 0–31 or `null`, `size`: `md`\|`lg`, `showValue` |
| `EmptyState` | `icon`, `title`, `description`, `children` (action) |
| `Spinner` | `size`, `label` |

Shared CSS helpers in `app.css`: `.page`, `.page--fill`, `.page--scroll`, `.page-header`,
`.page-title`, `.eyebrow`, `.mono`, `.num`, `.truncate`, `.stack`, `.row`, `.spread`,
`.metric-strip`, `.form-grid`, `.form-grid--single`, `.span-2`,
`.text-muted`, `.text-soft`, `.scroll-y`, `.sr-only`.

Tables of values use `font-variant-numeric: tabular-nums` (`.num`), never proportional
digits. Truncate inside flex/grid children with `min-width: 0` + `.truncate`.

## API contract

All through `$lib/api.js` (adds Basic auth, JSON encoding, 401 handling). Errors are
`ApiError` with `.status` and `.message`. `openConversationStream({onConversations,
onStatus})` returns an unsubscribe function.

Shapes returned by the Rust server:

```js
// GET /api/conversation  → sorted by timestamp desc
{ contact: { id, name }, sms_preview: { message, timestamp, status, sim_id } }

// GET /api/sms?page=&per_page=&contact_id=  (page 1 marks the thread read)
{ data: [{ id, contact_id, timestamp, message, sim_id, send, status }], total, page, per_page }
// send: true = outbound, false = inbound. status: 0 unread, 1 read, 2 loading, 3 failed

// GET /api/sim-cards
[{ id, imsi, phone_number, alias, created_at, updated_at }]

// GET /api/sims/info
[{
  sim_id, name, com_port, baud_rate,
  signal_quality: { rssi, ber } | null,
  operator_info: { operator_name, operator_id, registration_status } | null,
  model_info: { model } | null,
  sms_center: string | null,
  sim_status: string | null,      // e.g. "READY"
  memory_status: string | null,   // "Read: 5/100, Write: 5/100, Receive: 5/100"
  health_status: 'healthy'|'degraded'|'recovering'|'critical'|null,
  failure_reasons: string[] | null,
  consecutive_failures: number | null,
  last_probe_at: ISO string | null,
  last_ok_at: ISO string | null,
  last_recovery_action: string | null,
  last_recovery_at: ISO string | null
}]

// GET /api/sims/{id}/storage → { status: string | null }
// PUT /api/sims/{id}/storage { storage: 'SIM' | 'ME' | 'MT' }
// POST /api/sims/{id}/at { command } →
{ command, output, status: 'ok'|'error'|'unknown'|'transport_error', executed_at, duration_ms }
// GET /api/sims/{id}/refresh triggers device import/cleanup (204)
// PUT /api/sim-cards/{id}/alias { alias } | .../phone { phone_number } → SimCard
// DELETE /api/contacts/{id} deletes conversation history
```

`timestamp` values are naive local strings (`2026-10-21T09:30:00`); `$lib/format.js`
parses them as local time.

## Screen contracts

### Workspace — `(app)/+page.svelte`

Fill-height page with a conversation list (fixed 300–330px) and a message thread
(flexible). Each region has one heading and a hairline border; headers share the same
height. The thread scrolls to the newest message and paginates older messages upward
(`per_page: 30`). Composer sits at the bottom of the thread column with a line selector,
textarea, and send button aligned on one row, and only exists while a conversation is
open. The thread header carries contact identity, the line used and the actions (refresh
history, delete conversation with confirmation) — nothing else. No conversation selected
→ one-line empty state pointing at the inbox "+" action. Messages use soft 16px rounded
bubbles with very light neutral fills (inbound `--bubble-in`, outbound `--bubble-out`), no
border, and a small upward hook on the left (inbound) or right (outbound) in the upper
third, built from the supplied canvas template (its 171×93 canvas and 21px label are
scaled to our 13.5px type: hook top 27%, return 48%, protrusion 20%, micro-rounded tip).
The shape is drawn by `MessageBubble` on a DPR-aware canvas
(rounded rectangle + hook as one anti-aliased path) so the curve stays identical at every
bubble height and browser; text stays in normal HTML above the canvas. List updates arrive through the SSE stream and on send. On narrow container widths
(< 900px) the list collapses to a selection state; no column overlap or horizontal scroll.

### Fleet — `(app)/fleet/+page.svelte`

`PageHeader` + a compact `MetricStrip` (at most four metrics) + equal-height line card
grid + loading/empty/error states. Cards are `<a>` links to `/fleet/{sim_id}` and show
only: line number, display name, health, signal, operator, one identifier. No search box
for a short list, no per-card action menu (refresh is an action on the line detail
screen). Status filter, if present, is a small underline tab row, not a block selector.

### Line detail — `(app)/fleet/[simId]/+page.svelte`

Back link, `PageHeader` (display name, one identifier, health + signal in meta, actions:
refresh inbox, edit identity), then a two column body: left = health + storage + device
facts; right = AT console (wider). Each fact appears once: do not repeat the operator or
model in a metric strip and again in a facts list. AT console: quick command buttons that
execute immediately (at least AT, AT+CSQ, AT+COPS?, AT+CPIN?, AT+CREG?, AT+QTEMP?,
AT+CGSN, AT+CGMM), command input, run button, scrollable transcript with
status/duration/timestamp, copy and clear actions. Storage control writes `SIM|ME|MT` and
re-reads usage. Edit identity dialog updates alias and phone.
