## Overview
We will teach you how to quickly create your own DAO template.
## ***Your Job For daos modules***
> Implement the following types.
### daos-create-dao module
#### 1. `ConcreteId`
If you want to create a DAO template for a group, it's important to first analyze what brings people together.
In this group, people share one common characteristic,
This must be a characteristic that is not shared by any of the outside groups. Like having the same assets,
or having the same kind of NFT,
or being in the same room. This common characteristic can be assigned an id,
distinct from other groups, in daos we call this `ConcreteId`, and this id describes this group in a way.
In the `daos-create-dao` module, we'll find code like this
```commandline
type ConcreteId: Parameter
			+ Member
			+ TypeInfo
			+ MaxEncodedLen
			+ Clone
			+ Copy
			+ Default
			+ AccountIdConversion<Self::AccountId>
			+ BaseCallFilter<<Self as pallet::Config>::Call>
			+ TryCreate<Self::AccountId, Self::DaoId, DispatchError>;
```
In this case, We need to focus on implementing `TryCreate DispatchError>` `AccountIdConversion` and `BaseCallFilter<::Call>`.
* `TryCreate<Self::AccountId, Self::DaoId, DispatchError>` What are the requirements for users to create a DAO based on your DAO template,
such as the requirement in the VC DAO template for the kico project that the DAO creator must be the asset creator.
* `AccountIdConversion<Self::AccountId>` Each DAO created based on your DAO template has its own account id,
which allows the DAO to perform all the same transactions as any normal user on the chain.
Of course, the premise is that the transaction is`BaseCallFilter<::Call>`allowed.
* `BaseCallFilter<<Self as pallet::Config>::Call>` Which on-chain transactions can a dao account execute.
***
#### 2. `CallId`
```commandline
/// Each Call has its own id.
		type CallId: Parameter
			+ Copy
			+ MaybeSerializeDeserialize
			+ TypeInfo
			+ MaxEncodedLen
			+ Default
			+ TryFrom<<Self as pallet::Config>::Call>;
```
We find that CallId comes from Call.
This makes it easy for daos internal members to vote on the Origin setting for each transaction,
allowing the origin of each Call in the DAO to change dynamically.
#### 3. `DaoId`
```commandline
type DaoId: Clone + Default + Copy + Parameter + Member + MaxEncodedLen + CheckedAdd + One;
```
Sometimes, people don't care about the specifics of a DAO, just its id. Analogically,
DaoId is the serial number, and `ConcreteId` is more like the gender of the person.
This allows the DAOs created based on different characteristics to be linked.

#### 4. AfterCreate
```commandline
type AfterCreate: AfterCreate<Self::AccountId, Self::DaoId>;
```
Things that the DAO does after it is created, such as in the kico project,
the VC DAO is created with the creator set to sudo account.
### square module

#### 1. Pledge

```commandline
type Pledge: Clone
			+ Default
			+ Copy
			+ Parameter
			+ Member
			+ Pledge<
				BalanceOf<Self>,
				Self::AccountId,
				Self::DaoId,
				Self::Conviction,
				Self::BlockNumber,
				DispatchError,
			>;
```

Normally, during the referendum, we will use the token as the vote weight and lock it,
and then agree when to unlock it. Here you can customize what you use to vote,
And decide what to do with it before and after the vote.。

#### 2. Conviction

```commandline
type Conviction: Clone
			+ Default
			+ Copy
			+ Parameter
			+ ConvertInto<Self::BlockNumber>
			+ ConvertInto<BalanceOf<Self>>;
```
The multiplier of vote amplification, which determines the weight of your vote and how long it has a negative effect on you.
## ***Your Job For Extra Calls***
Write calls code for your DAO that are not yet on the chain. click [kico project example](https://github.com/DICO-TEAM/dico-chain/blob/main/pallets/vc/src/lib.rs)

## ***Conclusion***
As a developer, you only need to implement the above types according to the characteristics of your DAO,
and write Calls code for your DAO that are not yet on the chain, and
the rest of the code can be directly copied from the [kico project implementation](https://github.com/DICO-TEAM/dico-chain/blob/main/runtime/tico/src/vc.rs). Now you have successfully created your own DAO template.


