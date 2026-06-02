import { resolveApiPath } from '@/utils/runtimePaths';

const SESSION_KEY = 'sos_analytics_session_id';
const EVENT_URL = resolveApiPath('/analytics/event');

function createSessionId() {
    return `s_${Date.now()}_${Math.random().toString(36).slice(2, 10)}`;
}

export function getAnalyticsSessionId() {
    if (typeof window === 'undefined') return 'anonymous';

    const existing = window.localStorage.getItem(SESSION_KEY);
    if (existing) return existing;

    const next = createSessionId();
    window.localStorage.setItem(SESSION_KEY, next);
    return next;
}

export function trackEvent(eventKey, meta = {}) {
    if (typeof window === 'undefined') return;
    if (!eventKey || typeof eventKey !== 'string') return;

    const payload = {
        sessionId: getAnalyticsSessionId(),
        eventKey,
        page: window.location.pathname,
        meta
    };

    try {
        if (navigator.sendBeacon) {
            const blob = new Blob([JSON.stringify(payload)], { type: 'application/json' });
            navigator.sendBeacon(EVENT_URL, blob);
            return;
        }
    } catch {
        // 回退到 fetch
    }

    fetch(EVENT_URL, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(payload),
        keepalive: true
    }).catch(() => {});
}
