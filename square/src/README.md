# ***Square Module***
***
## Overview
The square module is about referendums,
where everyone has the right to vote and they can execute all transactions supported in all DAOs.
This is the highest authority.
In other words,
all transactions that the agency can execute,
the square module can execute.
In this module, the minimum voting weight required for each external transaction referendum can be determined, which is also equivalent to Square Origin.
***
## All Calls
***
### For every call
* `set_min_vote_weight_for_every_call` Set origin for a specific call.
### For basic parameters
* `set_max_public_props` Set the maximum number of proposals at the same time.
* `set_launch_period` Set the referendum interval.
* `set_minimum_deposit` Set the minimum amount a proposal needs to stake.
* `set_voting_period` Set the voting length of the referendum.
* `set_rerserve_period` Set the length of time that can be unreserved.
* `set_enactment_period` Set the time to delay the execution of the proposal.

### For Voting
* `propose` initiate a proposal.
* `second` Others support initiating proposals.
* `open_table` Open a referendum.
* `vote_for_referendum` Vote for the referendum.
* `cancel_vote` Cancel a vote on a referendum.
* `enact_proposal` Vote and execute the transaction corresponding to the proposa.
* `unlock` Release the locked amount.
