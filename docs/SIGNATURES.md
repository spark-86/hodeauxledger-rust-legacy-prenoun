# How signatures work

## Author Signature

Author signs over Intent, expressing the desire to record the data.

Author signature is expressed as author_sig = author_key.sign( H( intent ) )

## Usher Signature

Usher validates, and adds Context as part of its attestation.

Usher signature is expressed as usher_sig = usher_key.sign( H( author_sig || context( at, x, y, z, refer ) ) )

## Quorum Signature

Quorum signature is simply witness to author's intent and usher's context VIA their signatures.

Quorum signature is expressed as quorum_sig = quorum_key.sign ( H( author_sig || usher_sig ) )
