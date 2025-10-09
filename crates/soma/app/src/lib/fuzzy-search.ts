/**
 * Simple fuzzy search implementation
 * Returns true if all characters in the query appear in the same order in the text
 */
export function fuzzyMatch(query: string, text: string): boolean {
  if (!query) return true;
  if (!text) return false;

  const queryLower = query.toLowerCase();
  const textLower = text.toLowerCase();

  let queryIndex = 0;
  let textIndex = 0;

  while (queryIndex < queryLower.length && textIndex < textLower.length) {
    if (queryLower[queryIndex] === textLower[textIndex]) {
      queryIndex++;
    }
    textIndex++;
  }

  return queryIndex === queryLower.length;
}

/**
 * Fuzzy search with scoring
 * Returns a score based on how well the query matches the text
 * Higher score = better match
 */
export function fuzzyScore(query: string, text: string): number {
  if (!query) return 1;
  if (!text) return 0;

  const queryLower = query.toLowerCase();
  const textLower = text.toLowerCase();

  // Exact match
  if (textLower === queryLower) return 1000;

  // Starts with query
  if (textLower.startsWith(queryLower)) return 100;

  // Contains exact query
  if (textLower.includes(queryLower)) return 50;

  // Fuzzy match
  let score = 0;
  let queryIndex = 0;
  let textIndex = 0;
  let consecutiveMatches = 0;

  while (queryIndex < queryLower.length && textIndex < textLower.length) {
    if (queryLower[queryIndex] === textLower[textIndex]) {
      queryIndex++;
      consecutiveMatches++;
      score += consecutiveMatches * 2; // Bonus for consecutive matches
    } else {
      consecutiveMatches = 0;
    }
    textIndex++;
  }

  // Only return score if all characters were found
  return queryIndex === queryLower.length ? score : 0;
}

/**
 * Filter and sort items by fuzzy search
 * @param items Array of items to search
 * @param query Search query
 * @param getSearchText Function to extract searchable text from each item
 * @returns Filtered and sorted array
 */
export function fuzzyFilter<T>(
  items: T[],
  query: string,
  getSearchText: (item: T) => string
): T[] {
  if (!query) return items;

  const scoredItems = items
    .map(item => ({
      item,
      score: fuzzyScore(query, getSearchText(item))
    }))
    .filter(({ score }) => score > 0)
    .sort((a, b) => b.score - a.score);

  return scoredItems.map(({ item }) => item);
}