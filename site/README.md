# ghostr.social landing page

Static landing page for [ghostr.social](https://ghostr.social), hosted on
Cloudflare as a [Worker with static assets](https://developers.cloudflare.com/workers/static-assets/).
Everything under `public/` is served as-is; there is no Worker script.

- `public/index.html`, `public/styles.css` — the page. Colours mirror
  `lib/shared/theme/app_tokens.dart`.
- `public/logo.svg`, `public/favicon.svg` — copies of `assets/branding/`.
- `public/_headers` — security and caching headers.
- `public/404.html` — served for unknown paths.
- `wrangler.jsonc` — Worker name, asset directory, and the
  `ghostr.social` / `www.ghostr.social` custom domains.

Requires Node 22 or newer (wrangler's floor).

## Local preview

```sh
make site-dev          # or: cd site && npm ci && npm run dev
```

## Deploy

Pushes to `main` that touch `site/**` deploy automatically through
`.github/workflows/deploy_site.yaml`. To deploy by hand:

```sh
make site-deploy       # or: cd site && npm ci && npm run deploy
```

Either path needs Cloudflare credentials:

- locally, `npx wrangler login` once;
- in CI, the `CLOUDFLARE_API_TOKEN` repository secret (an API token created
  from the **Edit Cloudflare Workers** template, scoped to this account and
  the `ghostr.social` zone).
