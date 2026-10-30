/**
 * App-facing expense tracking API: thin typed wrappers over the generated Hey API client.
 *
 * Money is integer minor units on the wire (paise/cents); this module converts
 * to/from display strings. Dates are `YYYY-MM-DD` (Postgres `date`), displayed
 * `dd/MM/yyyy`.
 */
import { client } from '$lib/api/index.js';
import type {
  CreateExpenseBody,
  CreateSourceBody,
  ErrorBody,
  ExpenseDto,
  ExpenseSourceDto,
  UpdateExpenseBody,
  UpdateSourceBody,
} from '$lib/api/index.js';

export type {
  CreateExpenseBody,
  CreateSourceBody,
  ExpenseDto,
  ExpenseSourceDto,
  UpdateExpenseBody,
  UpdateSourceBody,
};

export const EXPENSE_SOURCES_QUERY_KEY = ['expense-sources'] as const;
export const expensesQueryKey = (month: string) => ['expenses', month] as const;

/**
 * Brand data per source key: official brand color (`null` for Vercel, whose
 * mark is pure black/white by design) + the Simple-Icons glyph path.
 * Used by the brand tile and the source picker dots.
 */
export const SOURCE_BRANDS: Record<string, { color: string | null; path: string }> = {
  vercel: { color: null, path: 'm12 1.608 12 20.784H0Z' },
  gcp: {
    color: '#4285F4',
    path: 'M12.19 2.38a9.344 9.344 0 0 0-9.234 6.893c.053-.02-.055.013 0 0-3.875 2.551-3.922 8.11-.247 10.941l.006-.007-.007.03a6.717 6.717 0 0 0 4.077 1.356h5.173l.03.03h5.192c6.687.053 9.376-8.605 3.835-12.35a9.365 9.365 0 0 0-2.821-4.552l-.043.043.006-.05A9.344 9.344 0 0 0 12.19 2.38zm-.358 4.146c1.244-.04 2.518.368 3.486 1.15a5.186 5.186 0 0 1 1.862 4.078v.518c3.53-.07 3.53 5.262 0 5.193h-5.193l-.008.009v-.04H6.785a2.59 2.59 0 0 1-1.067-.23h.001a2.597 2.597 0 0 1 3.437-3.437l3.013-3.012A6.747 6.747 0 0 0 8.11 8.24c.018-.01.04-.026.054-.023a5.186 5.186 0 0 1 3.67-1.69z',
  },
  aws: {
    color: '#FF9900',
    path: 'M6.763 10.036c0 .296.032.535.088.71.064.176.144.368.256.576.04.063.056.127.056.183 0 .08-.048.16-.152.24l-.503.335a.383.383 0 0 1-.208.072c-.08 0-.16-.04-.239-.112a2.47 2.47 0 0 1-.287-.375 6.18 6.18 0 0 1-.248-.471c-.622.734-1.405 1.101-2.347 1.101-.67 0-1.205-.191-1.596-.574-.391-.384-.59-.894-.59-1.533 0-.678.239-1.23.726-1.644.487-.415 1.133-.623 1.955-.623.272 0 .551.024.846.064.296.04.6.104.918.176v-.583c0-.607-.127-1.03-.375-1.277-.255-.248-.686-.367-1.3-.367-.28 0-.568.031-.863.103-.295.072-.583.16-.862.272a2.287 2.287 0 0 1-.28.104.488.488 0 0 1-.127.023c-.112 0-.168-.08-.168-.247v-.391c0-.128.016-.224.056-.28a.597.597 0 0 1 .224-.167c.279-.144.614-.264 1.005-.36a4.84 4.84 0 0 1 1.246-.151c.95 0 1.644.216 2.091.647.439.43.662 1.085.662 1.963v2.586zm-3.24 1.214c.263 0 .534-.048.822-.144.287-.096.543-.271.758-.51.128-.152.224-.32.272-.512.047-.191.08-.423.08-.694v-.335a6.66 6.66 0 0 0-.735-.136 6.02 6.02 0 0 0-.75-.048c-.535 0-.926.104-1.19.32-.263.215-.39.518-.39.917 0 .375.095.655.295.846.191.2.47.296.838.296zm6.41.862c-.144 0-.24-.024-.304-.08-.064-.048-.12-.16-.168-.311L7.586 5.55a1.398 1.398 0 0 1-.072-.32c0-.128.064-.2.191-.2h.783c.151 0 .255.025.31.08.065.048.113.16.16.312l1.342 5.284 1.245-5.284c.04-.16.088-.264.151-.312a.549.549 0 0 1 .32-.08h.638c.152 0 .256.025.32.08.063.048.12.16.151.312l1.261 5.348 1.381-5.348c.048-.16.104-.264.16-.312a.52.52 0 0 1 .311-.08h.743c.127 0 .2.065.2.2 0 .04-.009.08-.017.128a1.137 1.137 0 0 1-.056.2l-1.923 6.17c-.048.16-.104.263-.168.311a.51.51 0 0 1-.303.08h-.687c-.151 0-.255-.024-.32-.08-.063-.056-.119-.16-.15-.32l-1.238-5.148-1.23 5.14c-.04.16-.087.264-.15.32-.065.056-.177.08-.32.08zm10.256.215c-.415 0-.83-.048-1.229-.143-.399-.096-.71-.2-.918-.32-.128-.071-.215-.151-.247-.223a.563.563 0 0 1-.048-.224v-.407c0-.167.064-.247.183-.247.048 0 .096.008.144.024.048.016.12.048.2.08.271.12.566.215.878.279.319.064.63.096.95.096.502 0 .894-.088 1.165-.264a.86.86 0 0 0 .415-.758.777.777 0 0 0-.215-.559c-.144-.151-.416-.287-.807-.415l-1.157-.36c-.583-.183-1.014-.454-1.277-.813a1.902 1.902 0 0 1-.4-1.158c0-.335.073-.63.216-.886.144-.255.335-.479.575-.654.24-.184.51-.32.83-.415.32-.096.655-.136 1.006-.136.175 0 .359.008.535.032.183.024.35.056.518.088.16.04.312.08.455.127.144.048.256.096.336.144a.69.69 0 0 1 .24.2.43.43 0 0 1 .071.263v.375c0 .168-.064.256-.184.256a.83.83 0 0 1-.303-.096 3.652 3.652 0 0 0-1.532-.311c-.455 0-.815.071-1.062.223-.248.152-.375.383-.375.71 0 .224.08.416.24.567.159.152.454.304.877.44l1.134.358c.574.184.99.44 1.237.767.247.327.367.702.367 1.117 0 .343-.072.655-.207.926-.144.272-.336.511-.583.703-.248.2-.543.343-.886.447-.36.111-.734.167-1.142.167zM21.698 16.207c-2.626 1.94-6.442 2.969-9.722 2.969-4.598 0-8.74-1.7-11.87-4.526-.247-.223-.024-.527.272-.351 3.384 1.963 7.559 3.153 11.877 3.153 2.914 0 6.114-.607 9.06-1.852.439-.2.814.287.383.607zM22.792 14.961c-.336-.43-2.22-.207-3.074-.103-.255.032-.295-.192-.063-.36 1.5-1.053 3.967-.75 4.254-.399.287.36-.08 2.826-1.485 4.007-.215.184-.423.088-.327-.151.32-.79 1.03-2.57.695-2.994z',
  },
  cloudflare: {
    color: '#F38020',
    path: 'M16.5088 16.8447c.1475-.5068.0908-.9707-.1553-1.3154-.2246-.3164-.6045-.499-1.0615-.5205l-8.6592-.1123a.1559.1559 0 0 1-.1333-.0713c-.0283-.042-.0351-.0986-.021-.1553.0278-.084.1123-.1484.2036-.1562l8.7359-.1123c1.0351-.0489 2.1601-.8868 2.5537-1.9136l.499-1.3013c.0215-.0561.0293-.1128.0147-.168-.5625-2.5463-2.835-4.4453-5.5499-4.4453-2.5039 0-4.6284 1.6177-5.3876 3.8614-.4927-.3658-1.1187-.5625-1.794-.499-1.2026.119-2.1665 1.083-2.2861 2.2856-.0283.31-.0069.6128.0635.894C1.5683 13.171 0 14.7754 0 16.752c0 .1748.0142.3515.0352.5273.0141.083.0844.1475.1689.1475h15.9814c.0909 0 .1758-.0645.2032-.1553l.12-.4268zm2.7568-5.5634c-.0771 0-.1611 0-.2383.0112-.0566 0-.1054.0415-.127.0976l-.3378 1.1744c-.1475.5068-.0918.9707.1543 1.3164.2256.3164.6055.498 1.0625.5195l1.8437.1133c.0557 0 .1055.0263.1329.0703.0283.043.0351.1074.0214.1562-.0283.084-.1132.1485-.204.1553l-1.921.1123c-1.041.0488-2.1582.8867-2.5527 1.914l-.1406.3585c-.0283.0713.0215.1416.0986.1416h6.5977c.0771 0 .1474-.0489.169-.126.1122-.4082.1757-.837.1757-1.2803 0-2.6025-2.125-4.727-4.7344-4.727',
  },
  upstash: {
    color: '#00D9A3',
    path: 'M13.8027 0C11.193 0 8.583.9952 6.5918 2.9863c-3.9823 3.9823-3.9823 10.4396 0 14.4219 1.9911 1.9911 5.2198 1.9911 7.211 0 1.991-1.9911 1.991-5.2198 0-7.211L12 12c.9956.9956.9956 2.6098 0 3.6055-.9956.9955-2.6099.9955-3.6055 0-2.9866-2.9868-2.9866-7.8297 0-10.8164 2.9868-2.9868 7.8297-2.9868 10.8164 0l1.8028-1.8028C19.0225.9952 16.4125 0 13.8027 0zM12 12c-.9956-.9956-.9956-2.6098 0-3.6055.9956-.9955 2.6098-.9955 3.6055 0 2.9867 2.9868 2.9867 7.8297 0 10.8164-2.9867 2.9868-7.8297 2.9868-10.8164 0l-1.8028 1.8028c3.9823 3.9822 10.4396 3.9822 14.4219 0 3.9823-3.9824 3.9823-10.4396 0-14.4219-.9956-.9956-2.3006-1.4922-3.6055-1.4922-1.3048 0-2.6099.4966-3.6054 1.4922-1.9912 1.9912-1.9912 5.2198 0 7.211z',
  },
  neon: {
    color: '#00E599',
    path: 'M24 0V24l-9.365-8.045V24H0V0ZM2.942 21.087h8.751V9.563l9.365 8.204V2.919L2.942 2.914Z',
  },
  openrouter: {
    color: '#94A3B8',
    path: 'M16.778 1.844v1.919q-.569-.026-1.138-.032-.708-.008-1.415.037c-1.93.126-4.023.728-6.149 2.237-2.911 2.066-2.731 1.95-4.14 2.75-.396.223-1.342.574-2.185.798-.841.225-1.753.333-1.751.333v4.229s.768.108 1.61.333c.842.224 1.789.575 2.185.799 1.41.798 1.228.683 4.14 2.75 2.126 1.509 4.22 2.11 6.148 2.236.88.058 1.716.041 2.555.005v1.918l7.222-4.168-7.222-4.17v2.176c-.86.038-1.611.065-2.278.021-1.364-.09-2.417-.357-3.979-1.465-2.244-1.593-2.866-2.027-3.68-2.508.889-.518 1.449-.906 3.822-2.59 1.56-1.109 2.614-1.377 3.978-1.466.667-.044 1.418-.017 2.278.02v2.176L24 6.014Z',
  },
  'openai-platform': {
    // OpenAI's mark is monochrome by brand — inherits the foreground color.
    color: null,
    path: 'M22.2819 9.8211a5.9847 5.9847 0 0 0-.5157-4.9108 6.0462 6.0462 0 0 0-6.5098-2.9A6.0651 6.0651 0 0 0 4.9807 4.1818a5.9847 5.9847 0 0 0-3.9977 2.9 6.0462 6.0462 0 0 0 .7427 7.0966 5.98 5.98 0 0 0 .511 4.9107 6.051 6.051 0 0 0 6.5146 2.9001A5.9847 5.9847 0 0 0 13.2599 24a6.0557 6.0557 0 0 0 5.7718-4.2058 5.9894 5.9894 0 0 0 3.9977-2.9001 6.0557 6.0557 0 0 0-.7475-7.0729zm-9.022 12.6081a4.4755 4.4755 0 0 1-2.8764-1.0408l.1419-.0804 4.7783-2.7582a.7948.7948 0 0 0 .3927-.6813v-6.7369l2.02 1.1686a.071.071 0 0 1 .038.052v5.5826a4.504 4.504 0 0 1-4.4945 4.4944zm-9.6607-4.1254a4.4708 4.4708 0 0 1-.5346-3.0137l.142.0852 4.783 2.7582a.7712.7712 0 0 0 .7806 0l5.8428-3.3685v2.3324a.0804.0804 0 0 1-.0332.0615L9.74 19.9502a4.4992 4.4992 0 0 1-6.1408-1.6464zM2.3408 7.8956a4.485 4.485 0 0 1 2.3655-1.9728V11.6a.7664.7664 0 0 0 .3879.6765l5.8144 3.3543-2.0201 1.1685a.0757.0757 0 0 1-.071 0l-4.8303-2.7865A4.504 4.504 0 0 1 2.3408 7.872zm16.5963 3.8558L13.1038 8.364 15.1192 7.2a.0757.0757 0 0 1 .071 0l4.8303 2.7913a4.4944 4.4944 0 0 1-.6765 8.1042v-5.6772a.79.79 0 0 0-.407-.667zm2.0107-3.0231l-.142-.0852-4.7735-2.7818a.7759.7759 0 0 0-.7854 0L9.409 9.2297V6.8974a.0662.0662 0 0 1 .0284-.0615l4.8303-2.7866a4.4992 4.4992 0 0 1 6.6802 4.66zM8.3065 12.863l-2.02-1.1638a.0804.0804 0 0 1-.038-.0567V6.0742a4.4992 4.4992 0 0 1 7.3757-3.4537l-.142.0805L8.704 5.459a.7948.7948 0 0 0-.3927.6813zm1.0976-2.3654l2.602-1.4998 2.6069 1.4998v2.9994l-2.5974 1.4997-2.6067-1.4997Z',
  },
};

/** Brand color for a source key, or `null` when it has no brand color. */
export function sourceBrandColor(key: string): string | null {
  return SOURCE_BRANDS[key]?.color ?? null;
}

function errorMessage(error: unknown, status: number | undefined, fallback: string): string {
  if (error && typeof error === 'object') {
    const msg = (error as Partial<ErrorBody>).error;
    if (typeof msg === 'string' && msg.length > 0) return msg;
  }
  return status ? `${fallback} (HTTP ${status})` : fallback;
}

async function unwrap<T>(
  promise: Promise<{ data?: T; error?: unknown; response?: Response }>,
  fallback: string,
): Promise<T> {
  const { data, error, response } = await promise;
  if (error || data === undefined) throw new Error(errorMessage(error, response?.status, fallback));
  return data;
}

/* ---------------- sources ---------------- */

export async function listSources(): Promise<ExpenseSourceDto[]> {
  const res = await unwrap(client.sources.list(), 'Could not load expense sources');
  return res.sources;
}

export function createSource(body: CreateSourceBody): Promise<ExpenseSourceDto> {
  return unwrap(client.sources.create({ body }), 'Could not create expense source');
}

export function updateSource(key: string, body: UpdateSourceBody): Promise<ExpenseSourceDto> {
  return unwrap(client.sources.update({ path: { key }, body }), 'Could not update expense source');
}

export async function deleteSource(key: string): Promise<void> {
  const { error, response } = await client.sources.delete({ path: { key } });
  if (error || !response?.ok) {
    throw new Error(errorMessage(error, response?.status, 'Could not delete expense source'));
  }
}

/* ---------------- expenses ---------------- */

export async function listExpenses(month: string): Promise<ExpenseDto[]> {
  const res = await unwrap(client.expenses.list({ query: { month } }), 'Could not load expenses');
  return res.expenses;
}

export function createExpense(body: CreateExpenseBody): Promise<ExpenseDto> {
  return unwrap(client.expenses.create({ body }), 'Could not record expense');
}

export function updateExpense(id: number, body: UpdateExpenseBody): Promise<ExpenseDto> {
  return unwrap(client.expenses.update({ path: { id }, body }), 'Could not update expense');
}

export async function deleteExpense(id: number): Promise<void> {
  const { error, response } = await client.expenses.delete({ path: { id } });
  if (error || !response?.ok) {
    throw new Error(errorMessage(error, response?.status, 'Could not delete expense'));
  }
}

/** Attach an invoice file (the generated client mistypes the multipart body, so plain fetch). */
export async function uploadInvoice(id: number, file: File): Promise<ExpenseDto> {
  const form = new FormData();
  form.append('file', file, file.name);
  const response = await fetch(`/api/expenses/${id}/invoice`, { method: 'PUT', body: form });
  if (!response.ok) {
    const body = (await response.json().catch(() => null)) as Partial<ErrorBody> | null;
    throw new Error(errorMessage(body, response.status, 'Could not attach invoice'));
  }
  return (await response.json()) as ExpenseDto;
}

export async function deleteInvoice(id: number): Promise<ExpenseDto> {
  return unwrap(client.expenses.deleteInvoice({ path: { id } }), 'Could not remove invoice');
}

/** Blob download URL for an attached invoice (caller must `URL.revokeObjectURL`). */
export async function invoiceObjectUrl(id: number): Promise<{ url: string; filename: string }> {
  const response = await fetch(`/api/expenses/${id}/invoice`);
  if (!response.ok) throw new Error('Could not download invoice');
  const blob = await response.blob();
  const disposition = response.headers.get('content-disposition') ?? '';
  const filename = /filename="([^"]+)"/.exec(disposition)?.[1] ?? 'invoice';
  return { url: URL.createObjectURL(blob), filename };
}

/* ---------------- money ---------------- */

const inrFormat = new Intl.NumberFormat('en-IN', { style: 'currency', currency: 'INR' });
const usdFormat = new Intl.NumberFormat('en-US', { style: 'currency', currency: 'USD' });

/** `175000` paise → `₹1,750.00`. */
export function formatInr(paise: number): string {
  return inrFormat.format(paise / 100);
}

/** `2000` cents → `$20.00`. */
export function formatUsd(cents: number): string {
  return usdFormat.format(cents / 100);
}

/** `"1750"` / `"1,750.50"` rupees → paise, or `null` when unparseable. */
export function parseInrToPaise(input: string): number | null {
  const cleaned = input.replace(/[,₹\s]/g, '');
  if (!/^\d+(\.\d{1,2})?$/.test(cleaned)) return null;
  return Math.round(parseFloat(cleaned) * 100);
}

/** `"20"` / `"20.50"` dollars → cents, or `null` when unparseable. */
export function parseUsdToCents(input: string): number | null {
  const cleaned = input.replace(/[$,\s]/g, '');
  if (cleaned === '') return null;
  if (!/^\d+(\.\d{1,2})?$/.test(cleaned)) return null;
  return Math.round(parseFloat(cleaned) * 100);
}

/* ---------------- dates ---------------- */

/** `2026-10-05` → `05/10/2026` (built from parts — no timezone shift). */
export function formatDay(iso: string): string {
  const [y, m, d] = iso.split('-');
  if (!y || !m || !d) return iso;
  return `${d}/${m}/${y}`;
}

/** This month as `YYYY-MM` (the default month filter). */
export function currentMonth(): string {
  const now = new Date();
  return `${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, '0')}`;
}

/** `2026-10` → `October 2026`. */
export function monthLabel(month: string): string {
  const [y, m] = month.split('-').map(Number);
  if (!y || !m || m < 1 || m > 12) return month;
  return new Date(y, m - 1, 1).toLocaleString('en-US', { month: 'long', year: 'numeric' });
}

/** Shift `YYYY-MM` by `delta` months. */
export function shiftMonth(month: string, delta: number): string {
  const [y, m] = month.split('-').map(Number);
  const shifted = new Date(y, m - 1 + delta, 1);
  return `${shifted.getFullYear()}-${String(shifted.getMonth() + 1).padStart(2, '0')}`;
}

/** Today as `YYYY-MM-DD` (for the create form's date default). */
export function todayIso(): string {
  const now = new Date();
  const m = String(now.getMonth() + 1).padStart(2, '0');
  const d = String(now.getDate()).padStart(2, '0');
  return `${now.getFullYear()}-${m}-${d}`;
}

/* ---------------- aggregation ---------------- */

export interface SourceMonthTotal {
  source: ExpenseSourceDto;
  entries: ExpenseDto[];
  /** Sum of final INR charges, in paise. */
  totalInr: number;
  /** Sum of USD charges (present entries only), in cents, or `null` when none. */
  totalUsd: number | null;
  /** Distinct non-empty labels across entries. */
  labels: string[];
}

/** Group a month's entries by source (every known source gets a card, even empty ones). */
export function totalsBySource(
  sources: ExpenseSourceDto[],
  entries: ExpenseDto[],
): SourceMonthTotal[] {
  const byKey = new Map<string, ExpenseDto[]>();
  for (const entry of entries) {
    const list = byKey.get(entry.source_key) ?? [];
    list.push(entry);
    byKey.set(entry.source_key, list);
  }
  // The `misc` catch-all always renders last, after the branded vendors.
  const ordered = [...sources].sort((a, b) => (a.key === 'misc' ? 1 : b.key === 'misc' ? -1 : 0));
  return ordered.map((source) => {
    const list = byKey.get(source.key) ?? [];
    let totalInr = 0;
    let totalUsd = 0;
    let hasUsd = false;
    const labelSet = new Set<string>();
    for (const entry of list) {
      totalInr += entry.inr_paise;
      if (entry.usd_cents != null) {
        totalUsd += entry.usd_cents;
        hasUsd = true;
      }
      if (entry.label) labelSet.add(entry.label);
    }
    return {
      source,
      entries: list,
      totalInr,
      totalUsd: hasUsd ? totalUsd : null,
      labels: [...labelSet],
    };
  });
}
