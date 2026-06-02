export function clamp(v: number, min: number, max: number) {
  return Math.max(min, Math.min(max, v));
}

export function now() {
  return (performance && performance.now) ? performance.now() : Date.now();
}

export function normalizeText(s: string) {
  return (s ?? '').toString().trim().replace(/\s+/g, '');
}

export function pad2(n: number | string) {
  return String(n).padStart(2, '0');
}

export function getTodayISO() {
  const d = new Date();
  const y = d.getFullYear();
  const m = pad2(d.getMonth() + 1);
  const dd = pad2(d.getDate());
  return `${y}-${m}-${dd}`;
}

export function formatDateCNFromISO(iso: string) {
  // iso: YYYY-MM-DD
  const m = /^(\d{4})-(\d{2})-(\d{2})$/.exec(String(iso || ''));
  if (!m) return String(iso || '');
  return `${m[1]}年${m[2]}月${m[3]}日`;
}

function generateExamNoByDate(iso: string) {
  // 规律：BK + YYYYMMDD + "-" + 4位随机
  const ymd = String(iso || '').replace(/-/g, '');
  const rnd = String(Math.floor(Math.random() * 10000)).padStart(4, '0');
  return `BK${ymd}-${rnd}`;
}

export function getOrCreateExamNo(iso: string, prefixKey: string) {
  const keyBase = prefixKey || 'haruhi_exam_paper';
  const k = `${keyBase}__examno__${iso}`;
  try {
    const old = localStorage.getItem(k);
    if (old) return old;
    const v = generateExamNoByDate(iso);
    localStorage.setItem(k, v);
    return v;
  } catch (e) {
    return generateExamNoByDate(iso);
  }
}