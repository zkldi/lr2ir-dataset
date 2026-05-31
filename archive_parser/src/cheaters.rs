use std::collections::HashSet;

/// LR2IR player IDs flagged as cheaters.
///
/// These are just the "obvious" cheaters. I'm not taking a strong stand on things like double keys
/// or anything like that. These are just people that are actively fucking with every leaderboard.
///
/// RIP JADONG <3
pub const CHEATER_IDS: &[i32] = &[
	// https://github.com/yeslyko/goodrating/blob/master/goodrating.cpp#L43
	122738, // JADONG_GOD
	114328, // JADONG
	159674, // meumeu7
	162280, // Chieri-Kata
	111023, // 不正
	108312, // SS Officer
	113338, // FUGAGOOD
	104837, // FUGAFUCK
	153667, // 0133
	141249, // Ta2
	142961, // Amluox
	145628, // OJ.Amluox
	139857, // Pazo
	111571, // AiLee
	183696, // zionfan
	144372, // BMS KING
];

pub fn cheater_set() -> HashSet<i64> {
	CHEATER_IDS.iter().map(|&id| i64::from(id)).collect()
}

pub fn is_cheater_id(player_id: i64, cheaters: &HashSet<i64>) -> i64 {
	i64::from(cheaters.contains(&player_id))
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn empty_list_marks_nobody() {
		let cheaters = cheater_set();
		assert_eq!(is_cheater_id(123, &cheaters), 0);
	}

	#[test]
	fn listed_id_is_flagged() {
		let mut cheaters = HashSet::new();
		cheaters.insert(123);
		assert_eq!(is_cheater_id(123, &cheaters), 1);
		assert_eq!(is_cheater_id(321, &cheaters), 0);
	}
}
