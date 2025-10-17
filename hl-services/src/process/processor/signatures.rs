use hl_core::{
    Key, Policy, Rhex, Signature, error, keymaster::keymaster::Keymaster, policy::rule::Rule,
    rhex::signature::SigType, to_base64,
};
use hl_io::db;
use rusqlite::Connection;

use crate::process::processor::errors::Errors;

pub fn signature_usher_and_quorum(
    rhex: &mut Rhex,
    errors: &mut Errors,
    keymaster: &Keymaster,
) -> Result<(), anyhow::Error> {
    // First check to make sure we don't have some sig bullshit going
    // on. Should be author only.
    if rhex.signatures[0].sig_type != SigType::Author {
        errors.push(error::E_SIG_INVALID, "Signature must be of type Author");
        return Ok(());
    }
    // Add context to the R⬢
    rhex.add_context(&None)?;
    // Next, sign as selected usher.
    let usher_pk = rhex.intent.usher_pk;
    let usher_key = Key::from_bytes(keymaster.get_matching(&usher_pk).unwrap());
    let usher_sig = usher_key.sign(&rhex.usher_hash(&rhex.signatures[0].clone())?);
    rhex.signatures.push(Signature {
        sig_type: SigType::Usher,
        public_key: usher_pk,
        sig: usher_sig.unwrap(),
    });

    // Finally sign as quorum
    // TODO: Eventually we want this to be a policy option or something
    // that dictates if the signing usher also signs quorum, but for now
    // we assume we need to sign our own quorum
    let quorum_sig = usher_key
        .sign(&rhex.quorum_hash(&rhex.signatures[0].clone(), &rhex.signatures[1].clone())?);
    rhex.signatures.push(Signature {
        sig_type: SigType::Quorum,
        public_key: usher_pk,
        sig: quorum_sig.unwrap(),
    });

    Ok(())
}

pub fn signature_quorum(
    rhex: &mut Rhex,
    errors: &mut Errors,
    keymaster: &Keymaster,
) -> Result<(), anyhow::Error> {
    // Make sure the sigs aren't all fucked up. Author should always
    // be first, followed by Usher.
    if rhex.signatures[0].sig_type != SigType::Author {
        errors.push(error::E_SIG_INVALID, "Signature 0 must be of type Author");
        return Ok(());
    }
    if rhex.signatures[1].sig_type != SigType::Usher {
        errors.push(error::E_SIG_INVALID, "Signature 1 must be of type Usher");
        return Ok(());
    }

    // FIXME: This needs to load quorum members from policy (cache, most likely)
    // so we are making sure we are quorum member.

    // Finally sign as quorum
    // TODO: Eventually we want this to be a policy option or something
    // that dictates if the signing usher also signs quorum, but for now
    // we assume we need to sign our own quorum
    let usher_pk = rhex.intent.usher_pk;
    let usher_key = Key::from_bytes(keymaster.get_matching(&usher_pk).unwrap());
    let quorum_sig = usher_key
        .sign(&rhex.quorum_hash(&rhex.signatures[0].clone(), &rhex.signatures[1].clone())?);
    rhex.signatures.push(Signature {
        sig_type: SigType::Quorum,
        public_key: usher_pk,
        sig: quorum_sig.unwrap(),
    });
    Ok(())
}

pub fn signature_check_quorum(
    rhex: &Rhex,
    errors: &mut Errors,
    cache: &Connection,
) -> Result<bool, anyhow::Error> {
    // Check if there is a quorum signature
    let mut quorum_sigs = Vec::new();
    for signature in rhex.signatures.iter() {
        if signature.sig_type == SigType::Quorum {
            quorum_sigs.push(signature);
        }
    }

    if quorum_sigs.is_empty() {
        errors.push(error::E_SIG_INVALID, "No quorum signatures found");
        return Ok(false);
    }

    // Get the list of quorum members from policy
    let policy = db::policy::retrieve_policy(cache, &rhex.intent.scope);
    let mut policy = if policy.is_err() {
        let mut rule = Rule::new(&rhex.intent.scope);
        rule.record_types = vec!["scope:genesis".to_string()];
        rule.append_roles = vec!["authority".to_string()];
        rule.quorum_k = 1;
        rule.quorum_roles = vec!["authority".to_string()];
        rule.rate_per_mark = 1;
        let mut policy = Policy::new();
        policy.scope = rhex.intent.scope.clone();
        policy.quorum_ttl = 1000000000;
        policy.rules = vec![rule];

        policy
    } else {
        policy.unwrap()
    };
    let rules = db::rule::get_rules(cache, &rhex.intent.scope)?;
    if rules.len() == 0 {
        errors.push(error::E_POLICY_INVALID, "No rules found for this scope");
        return Ok(false);
    } else {
        policy.rules = rules;
    }
    let mut done = false;
    let mut da_rule = Rule::new(&rhex.intent.scope);
    for rule in policy.rules.iter() {
        if rule.applies_to(&rhex.intent.record_type) || rule.record_types == vec!["defaults"] {
            // We found the rule that applies to this record type
            // Now we need to check the quorum requirements

            da_rule.record_types = rule.record_types.clone();
            da_rule.append_roles = rule.append_roles.clone();
            da_rule.quorum_k = rule.quorum_k;
            da_rule.quorum_roles = rule.quorum_roles.clone();
            da_rule.rate_per_mark = rule.rate_per_mark;

            done = true;
        }
    }

    if !done {
        errors.push(
            error::E_POLICY_INVALID,
            "No applicable rule found for this record type",
        );
        return Ok(false);
    }

    // Does our sig count match policy quorum for this record_type?
    let quorum_k = da_rule.quorum_k; // Default to 1 for now, will come from policy
    if (quorum_sigs.len() as u16) < quorum_k {
        errors.push(
            error::E_QUORUM_INSUFFICIENT,
            format!(
                "Quorum not met. Expected at least {} signatures, but got {}",
                quorum_k,
                quorum_sigs.len()
            ),
        );
        return Ok(false);
    }

    // Make sure each sig is in the list of quorum members
    let authorities = db::authority::get_authorities(&cache, &rhex.intent.scope)?;
    for sig in quorum_sigs.iter() {
        let quorum_pk = sig.public_key;
        for auth in authorities.iter() {
            if auth.key.pk == Some(quorum_pk) {
                // Found a matching authority, break and continue to the next signature

                // Check if the authority has the required role for quorum
                let mut has_role = false;
                for role in da_rule.quorum_roles.iter() {
                    if auth.roles.contains(role) {
                        has_role = true;
                        break;
                    }
                }
                if !has_role {
                    errors.push(
                        error::E_QUORUM_INVALID_MEMBER,
                        format!(
                            "Quorum member {} does not have required role for record type {}",
                            to_base64(&quorum_pk),
                            rhex.intent.record_type
                        ),
                    );
                    return Ok(false);
                }

                // Verify signature while we are here
                let author_sig = rhex
                    .signatures
                    .get(0)
                    .ok_or_else(|| anyhow::anyhow!("Author signature missing for quorum check"))?;
                let usher_sig = rhex
                    .signatures
                    .get(1)
                    .ok_or_else(|| anyhow::anyhow!("Usher signature missing for quorum check"))?;
                let quorum_hash = rhex.quorum_hash(author_sig, usher_sig)?;
                if !auth.key.verify(&quorum_hash, &sig.sig)? {
                    errors.push(
                        error::E_SIG_INVALID,
                        format!("Quorum signature from {} is invalid", to_base64(&quorum_pk)),
                    );
                    return Ok(false);
                }

                break;
            }
        }
    }

    Ok(true)
}
