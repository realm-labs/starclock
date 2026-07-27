# Goal 07 Service Partition S01

`G07-P4-M14-S01` executes the first Standard Simulated Universe service
partition. It owns 16 records, 16 rules and nine semantic fixtures covering
Cosmic Fragments, Downloader, Blessing enhancement and reset, Respite offers,
Reviver, nine authored shop stages and the first Trailblaze Bonus.

`Universe.xlsx`, `UniverseBindings.xlsx` and `UniverseEvidence.xlsx` remain
the editable authority. An openpyxl verifier reads the recreated workbooks,
rejects formulas and broken provenance, checks contiguous service parameters,
and compares the selected rows with Sora 0.3.0 debug JSON. The partition
golden records the workbook, Sora bundle and semantic table digests.

## Currency and fixed prices

Cosmic Fragments are an Activity-scoped bounded integer. Standard runs begin
with 50 before any Ability Tree contribution. Service settlement uses checked
integer operations and rejects an unaffordable purchase without changing
state, RNG counters or the pending offer.

The public fixed values represented by this partition are:

- Blessing enhancement: 100, 130 or 160 fragments for a one-, two- or
  three-star Blessing;
- post-battle Blessing reset: 30, 50 and 100 fragments in schedule order;
- Respite: one one-star Blessing for 80, one Curio for 120, or two random
  Blessing enhancements for 180;
- Reviver: 80 fragments for one defeated participant restored to 100% HP;
- Downloader: one selected reserve participant with no fragment charge.

Independent Curio cost modifiers are composed at the service debit boundary.
IPC Cuckoo Clock applies its 125% Blessing-service inflation before Faith
Bond applies its discount. Neither modifier affects Reviver or Curio
purchases.

## Spatial-free interaction boundary

Shops, Downloader, Trailblaze Bonus and offer selection are external Activity
decisions. The runtime supplies a stable ordered legal-command collection;
the caller supplies one selected command. A renderer may present a shop,
device or room, but no 3D position, collision or UI state enters the
authoritative model.

All nine authored shop identifiers compile to the same generic checked
service protocol. Their stage and pool select the visible offer externally.
The purchase command binds content, final cost and offer digest. The handler
then validates and atomically debits currency, grants the Blessing or Curio,
initializes Curio lifecycle state, increments service usage and consumes the
offer. Invalid or stale commands preserve exact state bytes.

Downloader and Trailblaze Bonus ID `1` use the same external-outcome boundary
because the caller must choose or provide information outside the combat
resolver. ID `1` is the first of the six Standard entry choices: it grants
exactly 100 Cosmic Fragments. The grant composes with the configured
50-fragment initial balance and any Ability Tree contribution, settles as a
checked Activity operation and is replay-recorded.

## Cross-battle revival

Reviver consumes the participant carry ledger produced by the ordinary
nested-battle result contract. The service is legal only when:

1. the Ability Tree projects Reviver authorization and a 100% restored-HP
   ratio;
2. the selected participant exists in carry with zero HP and a non-alive life
   state;
3. at least 80 Cosmic Fragments are available.

One Activity transaction checks all three conditions, debits 80 fragments,
records the service use and restores the participant to alive, present and
full HP. The production fixture starts a real prepared battle, settles a
defeated/departed participant, invokes the registered service handler and
commits its returned operations. It therefore proves the same carry path used
by a complete run and server-side replay verification.

## Runtime disposition

The partition uses only shared Activity primitives and the statically
registered generic service handler. No service-ID branch enters
`starclock-combat`, no native content handler is admitted, and no numeric
approximation is recorded. The service handler is an adapter from validated
typed payloads to generic Activity operations; Excel/Sora definitions retain
content ownership.
