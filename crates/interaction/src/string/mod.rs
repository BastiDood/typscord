#[cfg(test)]
mod tests;

/// Find the substring that contains the longest streak of the given character.
pub fn find_longest_streak(haystack: &str, needle: char) -> &str {
	let char_indices: Vec<(usize, char)> = haystack.char_indices().collect();
	let mut max_length = 0;
	let mut max_start = 0;
	let mut current_start = 0;
	let mut current_length = 0;

	for (i, (_, ch)) in char_indices.iter().copied().enumerate() {
		if ch == needle {
			if current_length == 0 {
				current_start = i;
			}
			current_length += 1;
			if current_length > max_length {
				max_length = current_length;
				max_start = current_start;
			}
		} else {
			current_length = 0;
		}
	}

	if max_length == 0 {
		// No matches found:
		// - Avoid slicing using `char_indices[max_start]` which would panic on empty input.
		// - Also prevents returning a non-empty slice when there was no match at all.
		return "";
	}

	let (start_byte, _) = char_indices[max_start];
	let end_byte = char_indices
		.get(max_start + max_length)
		// If the streak ends at the final char, there is no next entry; fall back to
		// `haystack.len()` as the exclusive end bound instead of panicking.
		.copied()
		.map(|(byte_pos, _)| byte_pos)
		.unwrap_or(haystack.len());

	&haystack[start_byte..end_byte]
}
