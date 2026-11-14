const yenFormatter = new Intl.NumberFormat("ja-JP", {
  style: "currency",
  currency: "JPY",
});

const dateFormatter = new Intl.DateTimeFormat("ja-JP");

export const formatYen = (amount: number) => yenFormatter.format(amount);

export const formatDate = (value: string) =>
  dateFormatter.format(new Date(value));

export function parseAppliedDate(input: string): string | null {
  const value = input.trim();
  if (!/^\d{4}-\d{2}-\d{2}$/.test(value)) return null;

  const date = new Date(`${value}T00:00:00.000Z`);
  if (Number.isNaN(date.getTime())) return null;

  const iso = date.toISOString();
  return iso.slice(0, 10) === value ? iso : null;
}
