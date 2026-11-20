# Split Roadmap

The plan for the split agreed in [#4](https://github.com/bea4dev/shoji-bar-4/issues/4): the
components in a library (`shoji-bar-4-core`), the layout in the binary (`shoji-bar-4`), and custom
shells as ordinary crates that depend on them. Each step is one PR into `split`.

![Layers of the UI and where each piece goes](../assets/hybrid_en.png)

The layers and `Part` / `Cx` / `Motion` follow the proposal in #4. Items marked with ★ go beyond that
proposal and are open for discussion.

## Target layout (from the proposal in #4)

```
shoji-bar-4-core/        components and machinery only; no layout decisions
├── context.rs           Cx, IslandState, Part
├── motion.rs            Motion (springs, delays, stagger)
├── layout.rs            row(), column(), spacer(): combine Parts
├── bar/
│   ├── mod.rs           Bar { left, center, right }, Bar::open(output)
│   ├── island.rs        Island: the frame (shape, padding, hover, click, highlight, entrance)
│   ├── clock.rs         time(), date()                       parts
│   ├── status.rs        wifi(), bluetooth(), volume(), battery()
│   └── workspaces.rs, window.rs, media.rs, tray.rs, launcher.rs
├── panels/              PanelDef and the contents of each panel
├── dock.rs, lock.rs, notify.rs, osd.rs, settings.rs, theme.rs, wm.rs, ...
└── run.rs               run(root): starts the shell, handles `msg` and `preview`

shoji-bar-4/             the shell as shipped: arranges the parts, sets the overall style
├── src/lib.rs           pub fn bar() -> Bar, pub fn motion() -> Motion   (the stock layout, public)
└── src/main.rs          shoji_bar_4_core::run(|| ...)

my-shell/                someone who wants their own shell: an ordinary crate that depends on
                         shoji-bar-4-core (and, optionally, shoji-bar-4 to start from the stock layout)
```

Four layers, each one customizable on its own:

```
Bar          where the islands go (left / center / right)
 └─ Island   the frame of an island: shape, padding, click, highlight, entrance animation
     └─ layout   how the parts inside are arranged (row / column)
         └─ Part     a single component: clock::time(), status::wifi(), ...
             └─ data     settings(), battery(), network(), ... (as today)
```

## Rules for every step

- One PR = one branch, started from an up to date `split`, with small atomic commits: each one builds
  and passes `cargo test --workspace`.
- Verify with preview and tests: a refactor must not change what is drawn, so
  `shoji-bar-4 preview bar` and `preview quick-settings` give the same image before and after.
- The PR text explains what moves, why, and how it is wired.

## Steps (PRs)

1. Refactor: core crate ([#12](https://github.com/bea4dev/shoji-bar-4/pull/12))
   - [x] make repo a cargo workspace
   - [x] public entry points for wayland layers
   - [x] move into shoji-bar-4-core
   - [x] call to run in the clean main
   - [x] fix docs describing workspace updating pointers to code
   - [x] only the binary is a default member (the core was included by mistake)

2. Refactor: stock layout as `bar()`
   - [x] re-export `anti_webui` and `anti_webui_md3e` from the core, so a shell depends on the core
         only (suggested by bea4dev in the review of #12)
   - [x] `shoji-bar-4` has a library target (`src/lib.rs`)
   - [x] `Bar { left, center, right }` in `bar/mod.rs`
   - [x] `Island { id, build }` in `bar/island.rs`, with `Island::new(id, build)`
   - [x] each island in its own file under `bar/`: `launcher.rs`, `workspaces.rs`, `window.rs`,
         `clock.rs`, `media.rs`, `tray.rs`, `status.rs`; the frame they share (`bar_island`) in
         `island.rs`
   - [x] every stock island is an `Island` with a stable, unique `id` (`launcher`, `workspaces`,
         `window`, `clock`, `media`, `tray`, `status`), built by the `island()` of its file
   - [x] `Bar::open(&self, output)` opens the `Bar` it is given, and `view` draws it instead of the
         fixed list
   - [x] the layout of the bar out of the core: `default_bar()` in `shoji-bar-4/src/lib.rs`, the
         stock bar (bea4dev's proposal calls it `bar()`), and `main.rs` hands it over
   - [x] `empty_bar()` next to it, as part of the API: a bar with no islands, to build one from
         scratch
   - [x] `preview bar` draws the bar of the shell: `run_with(bar, root)`, since the core cannot call
         `default_bar()` (open question for bea4dev: is this the way?)

3. Create `Part` and `Cx`
   - [x] `context.rs`: `Part = Rc<dyn Fn(&Cx) -> View>`
   - [x] `Cx { output, placement }`, `#[non_exhaustive]`, built with `Cx::new`: `island` is a placeholder
         until step 4 and `motion` until step 6
   - [x] `layout.rs`: `row()`, `column()`, `spacer()`, which combine Parts
   - [x] the content of every island becomes a `Part`: `Island` holds a `Part`, and `view` builds
         one `Cx` for all of them

4. Refactor: move content out of the Island
   - [ ] `Island` as the frame (shape, padding, hover, click, highlight, entrance); its content is a
         `Part`
   - [ ] `IslandState { hovered, active, index }` in `Cx`
   - [ ] each island file split into its parts: `clock::time()` and `date()`, `status::wifi()`,
         `bluetooth()`, `volume()` and `battery()`...

5. Feat: bar builder (stable ids)
   - [ ] `.insert_before(id, island)`
   - [ ] `.insert_after(id, island)`
   - [ ] `.replace(id, island)`
   - [ ] `.remove(id)`
   - [ ] ★ `Vec<Section>` and `.section()`, to expand or reduce the number of sections instead of
         the fixed left / center / right (public on API to user)

6. Refactor: motion
   - [ ] `motion.rs`: collect the static/constants animation (`fast_spatial()`, `flow()`, `GOING`...)
         into `Motion`; `Motion::default()` keeps today's values
   - [ ] `motion` joins `Cx` (`Rc<Motion>`); `Island` can hold an `Option<Motion>`
   - [ ] `pub fn motion() -> Motion` in `shoji-bar-4/src/lib.rs`, next to `default_bar()`
   - [ ] `enter_delay` and `stagger`: new behaviour, in a commit of their own
   - [ ] ★ named motions: `motion.register("fade", f)`

7. Refactor: panel
   - [ ] `panels/`: `PanelDef` and its builder, replacing the closed `Panel` enum; the contents of
         each panel become `Part`s
   - [ ] `Cx` outside the bar (`island: Option<IslandState>`)
   - [ ] the calendar cascade becomes `motion.cascade_step`, with a public `pop_in` / `cascade(...)`
         helper
   - [ ] ★ `grid(rows, cols, spans)` as a layout for panels

8. Documentation
   - [ ] add examples: a custom shell, as a crate that depends on the core
   - [ ] update `docs/development.md` for the new layout of the code
   - [ ] add dotfiles as templates

## Open for discussion

- ★ Sections in any number, instead of the fixed `left` / `center` / `right`.
- ★ `Section::free()`, whose children are placed with `.absolute()` for exact pixels.
- ★ Named motions.
- ★ A grid layout for panels.
- How `preview` gets the bar from the binary: `run_with(bar, root)` (step 2).
- The names `default_bar()` and `empty_bar()`, instead of the `bar()` of the proposal.

## Extra

1. Patch of AntiWebUI -> To OTHER REPOSITORY
   - [ ] zoom
   - [ ] execute command of notification (Open, Edit...)

2. From the PR #4, if bea4dev wants them, each in a PR of its own
   - [ ] `--version`, `-V` and `version`
   - [ ] safe mode (`--default`), to run the stock interface when a custom shell misbehaves

3. Features specific to one desktop (for example Omarchy): they use the API once it exists, so they
   come after the split, in an optional crate.
