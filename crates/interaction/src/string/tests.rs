#![cfg(test)]

use super::find_longest_streak;

#[test]
fn empty_string_returns_empty() {
	assert_eq!(find_longest_streak("", 'a'), "");
}

#[test]
fn no_match_returns_empty() {
	assert_eq!(find_longest_streak("bbbb", 'a'), "");
	assert_eq!(find_longest_streak("こんにちは", 'a'), "");
}

#[test]
fn single_char_match() {
	assert_eq!(find_longest_streak("a", 'a'), "a");
	assert_eq!(find_longest_streak("ba", 'a'), "a");
	assert_eq!(find_longest_streak("ab", 'a'), "a");
}

#[test]
fn ascii_simple_streaks() {
	assert_eq!(find_longest_streak("aaabaaa", 'a'), "aaa");
	assert_eq!(find_longest_streak("baaab", 'a'), "aaa");
	assert_eq!(find_longest_streak("aaa", 'a'), "aaa");
}

#[test]
fn streak_at_start_and_end() {
	assert_eq!(find_longest_streak("aaab", 'a'), "aaa");
	assert_eq!(find_longest_streak("baaa", 'a'), "aaa");
}

#[test]
fn multiple_equal_length_streaks_returns_first() {
	assert_eq!(find_longest_streak("aa bb aaa cc aaa dd", 'a'), "aaa");
	assert_eq!(find_longest_streak("bb aa cc aa dd", 'a'), "aa");
}

#[test]
fn interleaved_characters() {
	assert_eq!(find_longest_streak("ababab", 'a'), "a");
	assert_eq!(find_longest_streak("xayazaa", 'a'), "aa");
}

#[test]
fn unicode_basic_bmp() {
	// Japanese Hiragana 'あ'
	let s = "ああいあああう";
	assert_eq!(find_longest_streak(s, 'あ'), "あああ");
	assert_eq!(find_longest_streak(s, 'い'), "い");
	assert_eq!(find_longest_streak(s, 'う'), "う");
}

#[test]
fn unicode_multibyte_emoji() {
	// Pile of Poo U+1F4A9 (💩) is multi-byte in UTF-8
	let s = "💩💩x💩💩💩y💩";
	assert_eq!(find_longest_streak(s, '💩'), "💩💩💩");
}

#[test]
fn unicode_flag_regional_indicators_treated_per_char() {
	// Regional indicators are each a distinct char; flags are sequences of two indicators.
	// Our function operates on chars, not grapheme clusters. Ensure it handles char equality.
	let s = "🇺🇸🇺🇸x🇺🇸"; // Each flag is two chars, so this is 2+2 + x + 2
	// Searching for a single regional indicator letter component '🇺' (U+1F1FA) should find the longest run of that char.
	let needle = '🇺';
	let expected = "🇺"; // alternating with 🇸, so no two 🇺 in a row
	assert_eq!(find_longest_streak(s, needle), expected);
}

#[test]
fn unicode_combining_marks_do_not_affect_char_match() {
	// 'a' + COMBINING ACUTE ACCENT (U+0301) vs plain 'a'
	let a_plain = "a";
	let a_comb = "a\u{0301}";
	let s = format!("{}{}{}{}{}", a_comb, a_plain, a_plain, a_comb, a_plain);
	// We search for plain 'a' so only exact 'a' chars count, not the combining sequence.
	assert_eq!(find_longest_streak(&s, 'a'), "aaa");
}

#[test]
fn longest_at_end_boundary() {
	let s = "xyzzzz";
	assert_eq!(find_longest_streak(s, 'z'), "zzzz");
}

#[test]
fn longest_at_start_boundary() {
	let s = "zzzzxy";
	assert_eq!(find_longest_streak(s, 'z'), "zzzz");
}

#[test]
fn all_same_character() {
	let s = "bbbbbb";
	assert_eq!(find_longest_streak(s, 'b'), "bbbbbb");
}

#[test]
fn alternating_unicode_and_ascii() {
	let s = "éaééaaéééa"; // includes multibyte 'é'
	assert_eq!(find_longest_streak(s, 'é'), "ééé");
	assert_eq!(find_longest_streak(s, 'a'), "aa");
}

#[test]
fn zero_width_joiner_sequences_are_per_characters_not_graphemes() {
	// Family emoji sequences use ZWJ; ensure we match by char, not grapheme cluster.
	let family = "👩\u{200D}👩\u{200D}👧\u{200D}👦"; // Woman ZWJ Woman ZWJ Girl ZWJ Boy
	let s = format!("{}x{}", family, family);
	assert_eq!(find_longest_streak(&s, '\u{200D}'), "\u{200D}");
}

#[test]
fn different_needles() {
	let s = "--==**==--";
	assert_eq!(find_longest_streak(s, '-'), "--");
	assert_eq!(find_longest_streak(s, '='), "==");
	assert_eq!(find_longest_streak(s, '*'), "**");
}
