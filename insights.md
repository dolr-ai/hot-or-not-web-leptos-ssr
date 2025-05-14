In the following run I notice that:
- There's a [cache hit](https://github.com/dolr-ai/hot-or-not-web-leptos-ssr/actions/runs/14776334227/job/41485742220#step:9:101)
- `cargo lint` [uses the cache](https://github.com/dolr-ai/hot-or-not-web-leptos-ssr/actions/runs/14776334227/job/41485742220#step:17:29)
- But there are new artifacts [generated](https://github.com/dolr-ai/hot-or-not-web-leptos-ssr/actions/runs/14776334227/job/41485742220#step:18:200). Implying missing files in the cache.
- And finally cache was _incorrectly_ reported as [up-to-date](https://github.com/dolr-ai/hot-or-not-web-leptos-ssr/actions/runs/14776334227/job/41485742220#step:41:13). Confirming the missing files in the cache.

Next steps:
- Check if we manually invalidate the cache and run the pipeline twice, does the second run goes faster?
  - If yes, then we know swatinem/rust-cache _is_ caching all the files. We can then focus on invalidation checks.
  - If no, then we know swatinem/rust-cache _is_ ignoring some files. We can then focus on adding _all_ the files.
    - Figure out what exactly is missing in the cache.
      - can be done by running after the build and then before the build of next run and then running a diff between the trees
