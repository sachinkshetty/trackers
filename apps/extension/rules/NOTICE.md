# EasyPrivacy Attribution

The generated `easyprivacy-*.json` rules include a supported subset of EasyPrivacy:

- Source: https://easylist.to/easylist/easyprivacy.txt
- Authors: The EasyList authors (https://easylist.to/)
- Selected distribution license: `CC-BY-SA-3.0-or-later`

The importer intentionally compiles only pure domain-block rules matching
`||example.com^`. Unsupported Adblock syntax is skipped.

The browser extension splits generated rules into static rulesets. Chromium
guarantees an extension at least 30,000 static rules; availability above that
depends on other installed extensions.
