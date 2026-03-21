/**
 * Get human-readable time until quota reset.
 * @param {string} resetsAt - ISO datetime string for reset time
 * @returns {string} e.g. "5h 23min"
 */
export function getTimeUntilReset(resetsAt) {
  if (!resetsAt) return '';

  const now = new Date();
  const reset = new Date(resetsAt);
  const diff = reset - now;

  if (diff <= 0) return '0h 0min';

  const hours = Math.floor(diff / (1000 * 60 * 60));
  const minutes = Math.floor((diff % (1000 * 60 * 60)) / (1000 * 60));

  return `${hours}h ${minutes}min`;
}
