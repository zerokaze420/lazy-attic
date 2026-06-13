import zh from './zh';
import en from './en';

const translations = { zh, en } as const;
type Locale = keyof typeof translations;
type Translation = typeof zh;

let locale = $state<Locale>('zh');

const saved = typeof localStorage !== 'undefined' ? localStorage.getItem('attic.locale') : null;
if (saved === 'en' || saved === 'zh') {
  locale = saved;
} else if (typeof navigator !== 'undefined') {
  const lang = navigator.language.toLowerCase();
  if (lang.startsWith('zh')) {
    locale = 'zh';
  } else if (lang.startsWith('en')) {
    locale = 'en';
  }
}

export function getLocale() {
  return locale;
}

export function setLocale(l: Locale) {
  locale = l;
  if (typeof localStorage !== 'undefined') {
    localStorage.setItem('attic.locale', l);
  }
}

export function toggleLocale() {
  setLocale(locale === 'zh' ? 'en' : 'zh');
}

type NestedKeyOf<T, K extends keyof T = keyof T> = K extends string
  ? T[K] extends Record<string, any>
    ? `${K}.${NestedKeyOf<T[K]>}`
    : K
  : never;

type DotPath = NestedKeyOf<Translation>;

function getNested(obj: any, path: string): string {
  return path.split('.').reduce((cur, key) => cur?.[key], obj) ?? path;
}

export function t(key: DotPath, params?: Record<string, string | number>): string {
  const dict = translations[locale];
  let text: string = getNested(dict, key);
  if (params) {
    for (const [k, v] of Object.entries(params)) {
      text = text.replace(`{${k}}`, String(v));
    }
  }
  return text;
}

export function formatDate(dateStr: string): string {
  if (!dateStr) return '-';
  const date = new Date(dateStr);
  const opts: Intl.DateTimeFormatOptions = {
    year: 'numeric',
    month: 'short',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit'
  };
  const langTag = locale === 'zh' ? 'zh-CN' : 'en-US';
  return new Intl.DateTimeFormat(langTag, opts).format(date);
}

export function formatBytes(value: number): string {
  if (!Number.isFinite(value)) return '-';
  const units = locale === 'zh'
    ? ['B', 'KiB', 'MiB', 'GiB', 'TiB']
    : ['B', 'KiB', 'MiB', 'GiB', 'TiB'];
  let size = Math.max(0, value);
  let unit = 0;
  while (size >= 1024 && unit < units.length - 1) {
    size /= 1024;
    unit += 1;
  }
  return `${size >= 10 || unit === 0 ? size.toFixed(0) : size.toFixed(1)} ${units[unit]}`;
}

export function formatRetention(seconds: number | null | undefined): string {
  if (seconds === null || seconds === undefined) return t('globalDefault');
  if (seconds === 0) return t('permanent');
  const days = Math.round(seconds / 86400);
  if (days >= 1) return `${days} ${t('days')}`;
  return `${Math.max(1, Math.round(seconds / 3600))} ${t('hours')}`;
}
