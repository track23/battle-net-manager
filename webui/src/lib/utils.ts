type ClassValue = string | boolean | undefined | null | Record<string, boolean | undefined | null>

export function cn(...classes: ClassValue[]): string {
  return classes
    .flatMap((c) => {
      if (!c || typeof c === 'boolean') return []
      if (typeof c === 'string') return [c]
      return Object.entries(c).filter(([, v]) => !!v).map(([k]) => k)
    })
    .join(' ')
}

export const DEFAULT_GROUP_ID = 'default'
export const EXPANDED_GROUPS_KEY = 'expanded_groups'
export const DARK_MODE_KEY = 'dark_mode'

/** Format a Date as local time in RFC 3339 (matches chrono::Local::now() serialization) */
export function toLocalISOString(date: Date): string {
  const offset = -date.getTimezoneOffset()
  const sign = offset >= 0 ? '+' : '-'
  const absOffset = Math.abs(offset)
  const hh = String(Math.floor(absOffset / 60)).padStart(2, '0')
  const mm = String(absOffset % 60).padStart(2, '0')
  const y = date.getFullYear()
  const M = String(date.getMonth() + 1).padStart(2, '0')
  const d = String(date.getDate()).padStart(2, '0')
  const H = String(date.getHours()).padStart(2, '0')
  const Min = String(date.getMinutes()).padStart(2, '0')
  const S = String(date.getSeconds()).padStart(2, '0')
  return `${y}-${M}-${d}T${H}:${Min}:${S}${sign}${hh}:${mm}`
}

/** Sort comparator: descending by LastUsed (most recent first) */
export function compareByLastUsed<T extends { LastUsed: string }>(a: T, b: T): number {
  return new Date(b.LastUsed).getTime() - new Date(a.LastUsed).getTime()
}
