# ***Agency Module***
***
## Overview
Agency module is the power agency in DAO,
it can handle things that need to be decided quickly in DAO more efficiently.
Also, this module provides a method to set the Origin for each external transaction,
and the Agency executes the external transaction according to the Origin.
***
## All Calls
***
### For every call
* `set_ensure_origin_for_every_call` Set origin for a specific call.
### For some Storage
* `set_motion_duration` Set the length of time for voting on proposal.
* `set_max_proposals` Set a cap on the number of agency's proposals.
* `set_max_member` Set the maximum number of members in the agency.
### For Voting
* `execute` Dispatch a proposal from a member using the `Member` origin.
* `propose` Add a new proposal to either be voted on or executed directly.
* `vote` Add an aye or nay vote for the sender to the given proposal.
* `close` Close a vote that is either approved, disapproved or whose voting period has ended.
* `disapprove_proposal` The Root disapprove a proposal, close, and remove it from the system, regardless of its current state.
