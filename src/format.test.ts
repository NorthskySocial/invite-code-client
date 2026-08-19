import { describe, it, expect } from 'vitest';
import { formatDate, formatDateCsv } from './App';

describe('formatDate', () => {
  it('returns a dash for undefined input', () => {
    expect(formatDate(undefined)).toBe('-');
  });

  it('returns "Invalid Date" for an unparseable string', () => {
    expect(formatDate('not-a-date')).toBe('Invalid Date');
  });

  it('formats a valid date', () => {
    const result = formatDate('2026-01-25T08:02:05');
    expect(result).not.toBe('-');
    expect(result).not.toBe('Invalid Date');
    expect(result).toContain('2026');
  });
});

describe('formatDateCsv', () => {
  it('returns a dash for undefined input', () => {
    expect(formatDateCsv(undefined)).toBe('-');
  });

  it('returns "Invalid Date" for an unparseable string', () => {
    expect(formatDateCsv('not-a-date')).toBe('Invalid Date');
  });

  it('formats a valid local-time date as "YYYY-MM-DD HH:mm"', () => {
    expect(formatDateCsv('2026-01-25T08:02:05')).toBe('2026-01-25 08:02');
  });

  it('zero-pads month, day, hour, and minute', () => {
    expect(formatDateCsv('2026-03-04T05:06:07')).toBe('2026-03-04 05:06');
  });
});
