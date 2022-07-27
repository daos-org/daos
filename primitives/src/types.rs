use super::*;

/// Simple index type for proposal counting.
pub type ProposalIndex = u32;

/// A number of members.
///
/// This also serves as a number of voting members, and since for motions, each member may
/// vote exactly once, therefore also the number of votes for any given motion.
pub type MemberCount = u32;

pub type RealCallId = u32;

#[derive(PartialEq, Encode, Decode, RuntimeDebug, Clone, TypeInfo, MaxEncodedLen)]
pub enum Proportion<MemberCount> {
	MoreThan(MemberCount, MemberCount),
	AtLeast(MemberCount, MemberCount),
}

impl Default for Proportion<MemberCount> {
	fn default() -> Self {
		Self::MoreThan(1, 1)
	}
}

#[derive(PartialEq, Encode, Decode, RuntimeDebug, Clone, TypeInfo, Copy, MaxEncodedLen)]
pub enum DoAsEnsureOrigin<Pro, C> {
	Proportion(Pro),
	Member,
	Members(C),
	Root,
	NoPermission,
}

impl<Pro: Default, C: Default> Default for DoAsEnsureOrigin<Pro, C> {
	fn default() -> Self {
		Self::Root
	}
}
