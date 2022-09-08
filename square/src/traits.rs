#![cfg_attr(not(feature = "std"), no_std)]
use super::*;

pub trait Vote<VoteWeight, AccountId, DaoId, Convivtion, BlockNumber, DispatchError> {
	fn try_vote(
		&self,
		who: &AccountId,
		dao_id: &DaoId,
		conviction: &Convivtion,
	) -> result::Result<(VoteWeight, BlockNumber), DispatchError>;
	fn vote_end_do(
		&self,
		who: &AccountId,
		dao_id: &DaoId,
	) -> result::Result<(), DispatchError>;
}

pub trait CheckedVote<SecondId, DispatchError> {
	fn is_can_vote(&self, second_id: SecondId) -> result::Result<bool, DispatchError>;
}

pub trait ConvertInto<A> {
	fn convert_into(&self) -> A;
}
