# ***Emergency***
***
## Overview
The emergency module is used in emergency situations, such as DAO cannot run normally due to some factors.

Internal personnel with authority to deal with emergencies, or ExternalOrigin,
They can make a proposal. But their rights can only be used in relation to emergencies.

Emergency members can reject external proposals.
Anyone can reject internal proposals.
***
## All Calls

### For everyone
* `reject` Rejected an emergency proposal.
* `enact_proposal` Execute a transaction related to an emergency internal proposal.
### For DAO External
* `external_track` Externally initiated an emergency proposal.
### For DAO Emergency Members
* `internal_track` Member initiates an urgent proposal.
* `reject` Rejected an emergency external proposal.
* `enact_proposal` Execute a transaction related to an emergency proposal.
### For DAO Root Account
* `set_members` Set members who can make emergency proposals.
* `set_pledge` Set the amount that needs to be pledge for an emergency proposal.
