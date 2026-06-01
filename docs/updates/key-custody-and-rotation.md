# Signing Key Custody and Rotation

This document defines how rule-update signing keys are handled for the prototype
and for the first public beta.

## Prototype Workflow

- Keep the signing private key out of the repository.
- Store the prototype private key in an encrypted local secret store or an
  offline hardware-backed device.
- Assign a stable `key_id` to the private/public key pair and publish the public
  key alongside the update manifest trust store.
- Sign update manifests and bundle hashes only from the release machine or a
  controlled signing host.
- Record the manifest `key_id` in release notes so verification failures can be
  traced quickly.

## Public-Beta Requirements

- Move from the prototype key to a dedicated release key before public beta.
- Store the release private key in offline hardware or an equivalent restricted
  signing environment.
- Limit access to the release key to the smallest practical set of operators.
- Require a documented signing checklist before each release.
- Keep previous public keys in the trust store until the rotation window closes
  so older verified bundles continue to validate.

## Rotation Process

1. Generate a new key pair.
2. Publish the new public key and `key_id` in the trust store.
3. Update the signer to use the new key for newly produced bundles.
4. Keep the previous key available until the last bundle signed with it is no
   longer distributed.
5. Retire the old private key only after the trust store no longer needs it for
   verification.

## Revocation Process

- Remove the revoked `key_id` from the trust store in a signed trust-store
  update.
- Treat any bundle signed by the revoked key as untrusted after the revocation
  update is applied.
- Document the revocation reason and the effective cutoff date.

## Operational Checklist

- [ ] Confirm the `key_id` in the manifest matches the signing key used for the
      build.
- [ ] Verify the bundle hash before publishing the manifest.
- [ ] Confirm the signature validates against the published public key.
- [ ] Store an audit record for the signing run.
- [ ] Keep the previous public key available during the rotation window.
