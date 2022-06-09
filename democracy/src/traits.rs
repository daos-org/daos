#![cfg_attr(not(feature = "std"), no_std)]
use super::*;
use sp_std::vec;

pub trait Vote<VoteWeight, AccountId, SecondId, Convivtion, BlockNumber, DispatchError> {
	fn try_vote(
		&self,
		who: &AccountId,
		second_id: &SecondId,
		conviction: &Convivtion,
	) -> result::Result<(VoteWeight, BlockNumber), DispatchError>;
	fn vote_end_do(
		&self,
		who: &AccountId,
		second_id: &SecondId,
	) -> result::Result<(), DispatchError>;
}

pub trait CheckedVote<SecondId, DispatchError> {
	fn is_can_vote(&self, second_id: SecondId) -> result::Result<bool, DispatchError>;
}

pub trait ConvertInto<A> {
	fn convert_into(&self) -> A;
}
