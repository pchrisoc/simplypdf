# SimplyPDF download landing page

This directory is a **static site** intended to be served as the document root for `pdf.simplyhost.org`. It mirrors the typography, background, navigation, and restrained visual language of [SimplyHost](https://simplyhost.org). There is no build step or JavaScript.

## Deployment

Serve `website/index.html` and `website/style.css` at the root of `pdf.simplyhost.org` using your preferred static web server or static-hosting provider. Configure DNS and HTTPS separately; this repository change does not modify DNS, a reverse proxy, or a VM.

The primary SimplyPDF brand logo is loaded directly from the latest `main` version of `assets/app-icon/simplypdf.svg` in this repository, avoiding a stale copied logo. For independent/offline hosting, copy the logo into the site as part of deployment instead.

The platform links intentionally open [the latest GitHub release](https://github.com/pchrisoc/simplypdf/releases/latest), **not** unverified direct installer URLs. Once a release matrix has been verified, these can be replaced with platform-specific asset URLs. Do not claim that Windows, macOS, or Linux packages are available before verifying the release assets.

## Icon provenance

The Windows four-pane mark is an original CSS construction. The macOS (Apple) and Linux OS logos are loaded at runtime from [Simple Icons v15](https://github.com/simple-icons/simple-icons), whose SVG artwork is distributed as CC0-1.0; they remain third-party trademark identifiers and are used to identify supported platform categories, not endorsements. The logos are fetched from jsDelivr; that dependency can be removed by locally bundling properly attributed, hash-pinned files under the repository asset policy. The SimplyPDF logo is the owner's artwork already documented in `assets/app-icon/README.md` and `ATTRIBUTION.toml`.

## Validation

Open `index.html` and inspect at desktop and mobile widths. Verify the three OS cards point to the current GitHub releases page and that the live web host serves the CSS. A GitHub merge does not itself deploy the site.
