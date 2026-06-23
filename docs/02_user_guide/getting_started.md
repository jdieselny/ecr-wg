# Getting Started

**Tier:** git-tracked.
**Source:** AI-generated, user-reviewed
**Audience:** a new SE adopting Continuum for the first time, or Justin setting up Continuum on a new machine

This guide takes you from "never heard of Continuum" to "running my first productive session" in about 30 minutes. If you hit a problem along the way, see `troubleshooting.md`.

## What you'll need

Before starting, have these in place:

- **Claude Desktop app** installed (macOS, Windows, or Linux)
- **Claude Code** installed and working on the terminal
- **Git** installed (Git for Windows on Windows, native on macOS/Linux)
- **An SSH key** already registered with your GitHub account (or willingness to generate one)
- **Write access** to the Datacom-owned private GitHub org where the Continuum repo lives
- **A terminal** you're comfortable in (PowerShell on Windows, Terminal/iTerm on macOS)

About 20-30 minutes of focused time. Don't try to rush this on a busy morning; do it when you can pay attention.

## Step 1: Create your workspace folder

Decide where Continuum will live on your machine. The recommended location is directly under your user profile, at a dedicated path that's easy to remember.

On Windows:
```
C:\Users\<you>\DatacomWorkspace\
```

On macOS/Linux:
```
~/DatacomWorkspace/
```

Create the folder. Leave it empty for now.

Important: this folder will contain two subdirectories (`continuum/` and `continuum-local/`). Do NOT run `git init` at the `DatacomWorkspace/` level. That's a well-known failure mode that captures your entire workspace including the local tier.

## Step 2: Clone the tracked tier

From your `DatacomWorkspace/` folder, clone the Continuum repo:

```
cd ~/DatacomWorkspace
git clone git@github.com-datacom:<datacom-org>/continuum.git
```

If you don't have a host alias configured yet, use the standard URL and configure routing afterward (see Step 5).

After cloning, you should see:
```
DatacomWorkspace/
└── continuum/
    ├── .git/
    ├── .gitignore
    ├── README.md
    ├── SANITIZATION.md
    ├── personas/
    ├── rituals/
    ├── roadmap/
    ├── shared/
    └── docs/
```

## Step 3: Create the local tier

Sibling to `continuum/`, create an empty `continuum-local/` directory:

```
mkdir continuum-local
```

**Do not `git init` inside this directory. Ever.**

Populate the expected subdirectories (these are templates; you'll fill them as you work):
```
continuum-local/
├── personas/sev/active_state.md      (create empty or copy template)
├── collaborators/                    (populated when you hire personas)
├── shared/product_internal.md        (your internal-only product questions)
├── shared/internal_codenames.md      (platform codenames to scrub from customer-facing output)
├── shared/templates/assets/          (real .pptx, .docx, logo files)
├── roadmap/in_flight.md              (live tasks with customer names)
└── scratchpad/                       (session logs, dumps, notes)
```

If the tracked tier has a `templates/` directory under `continuum/shared/templates_spec/`, use those specs to shape your local `assets/` content. Templates live local because they contain real branded material; their specifications live tracked.

## Step 4: Install the Filesystem MCP in Claude Desktop

Continuum depends on Claude Desktop having scoped filesystem access to your workspace. Install Anthropic's official Filesystem MCP connector and point it at `DatacomWorkspace/` (not any narrower, and not wider).

Open Claude Desktop settings, navigate to MCP configuration, and add a filesystem server entry for your workspace path. Exact steps vary by Claude Desktop version; refer to Anthropic's MCP documentation if the UI has changed.

Verify by starting a chat and asking Claude: "List the files in C:\Users\<you>\DatacomWorkspace\continuum". You should see the directory tree. If you get a permission error, revisit your MCP configuration.

## Step 5: Configure git identity and SSH routing

Continuum commits must be attributed to your Datacom email, not a personal one. Set this per-repo, not globally:

```
cd ~/DatacomWorkspace/continuum
git config user.email "<your-datacom-email>"
git config user.name "<your-name>"
```

If you have multiple GitHub accounts on the machine, set up SSH host aliases to prevent accidental personal-account commits. In `~/.ssh/config`:

```
Host github.com-datacom
  HostName github.com
  User git
  IdentityFile ~/.ssh/id_ed25519_datacom
  IdentitiesOnly yes

Host github.com-personal
  HostName github.com
  User git
  IdentityFile ~/.ssh/id_ed25519_personal
  IdentitiesOnly yes
```

Then verify the Continuum remote uses the datacom alias:
```
git remote -v
```
You should see `git@github.com-datacom:<org>/continuum.git`. If it's plain `github.com`, update it:
```
git remote set-url origin git@github.com-datacom:<org>/continuum.git
```

Test the connection:
```
ssh -T git@github.com-datacom
```
It should say "Hi <your-datacom-username>!".

## Step 6: Verify workspace state (run POST)

POST (Pre-Operation Self-Test) is the first ritual in Continuum. It verifies the workspace is in a known-good state before loading context.

Open Claude Desktop. Start a new chat. Say:

> Run POST

SEV should execute the POST ritual per `continuum/rituals/post.md`. At minimum it checks:

- `continuum/` exists and is a git repo
- `continuum-local/` exists and is NOT a git repo
- Expected persona/shared/rituals files are present
- No obvious sanitization violations in recently-touched tracked files

If POST fails, the output will name the specific failure. Fix it before proceeding. For structural checks SEV can do via MCP, trust the output. For git-state checks SEV can't run (it has no shell), use your terminal or Mrs. Code.

## Step 7: Load Continuum

Once POST passes, say:

> Load Continuum

SEV reads the persona card, shared rules, active state, and roadmap. Expect an acknowledgment that lists what was loaded and a brief summary of what's in flight.

If the acknowledgment feels generic or skips files, try again. A proper load names specific active opportunities and mentions open items from the roadmap.

## Step 8: Try a small first task

Don't start with a customer proposal. Start with something low-stakes to verify the loop works end-to-end.

Examples of good first tasks:
- "Draft a LinkedIn post about why network admins don't know visibility layers exist."
- "Help me think through competitive framing against Garland for a healthcare customer."
- "Review my Q3 goals draft against the North Star missions."

Run the task. Look at the output. Check that it sounds like your voice, respects the hard writing rules (no em dashes, no AI tell phrases), and doesn't include any internal codenames or customer specifics that would leak if you posted it.

If something's off, say so. The system improves by being corrected.

## Step 9: Commit the session

When you're done, say:

> Commit this

SEV proposes a tiered changeset: tracked-tier changes (if any) and local-tier changes (if any). Review it. Approve or reject parts.

Copy the approved commit packet into Mrs. Code on your terminal. She audits it independently, catches anything SEV missed, and runs the actual git commands. Once she confirms the push, check `git log` and `git status` to verify.

If Mrs. Code flags a sanitization violation, that's the system working. Fix the violation, try again.

## Step 10: Read the daily workflow

Now that the loop works, `daily_workflow.md` shows how to use Continuum productively day-to-day. `trigger_phrases.md` is a one-page reference card worth keeping open.

## What "done" looks like

You know setup worked when:

- `git status` on `continuum/` shows a clean tree
- `git -C ~/DatacomWorkspace/continuum-local status` returns "not a git repository" (THIS IS CORRECT)
- Claude Desktop can read files in your workspace on request
- "Run POST" returns a pass
- "Load Continuum" acknowledges with specific active work
- Mrs. Code can commit and push tracked-tier changes

If all of that is true, you're running Continuum.

## Common first-run issues

**"SEV says it can't see any files"** — Filesystem MCP isn't installed or isn't pointed at the workspace. Revisit Step 4.

**"I see files but Load Continuum just gives a generic response"** — the persona card and shared rules files exist but may be incomplete. Open `continuum/personas/sev/persona.md` and `continuum/shared/brand.md`; verify they have content. If this is a fresh clone, they should; if this is your own new SE persona, you'll need to write them.

**"Mrs. Code says 'not a git repo' when I try to commit"** — you're in the wrong directory, or running commands against `continuum-local/` (which is correct to not be a repo). Commits only happen in `continuum/`.

**"Git rejects my push with an authentication error"** — SSH routing is wrong. Verify Step 5 and test with `ssh -T git@github.com-datacom`.

**"I accidentally initialized git at the `DatacomWorkspace/` level"** — stop. Do not `git add .`. See `troubleshooting.md` for recovery. Short version: delete the stray `.git` directory before it commits anything.

## Related docs

- `daily_workflow.md` — how to use Continuum day-to-day
- `trigger_phrases.md` — quick reference of all ritual triggers
- `handoff_to_mrs_code.md` — the commit workflow in detail
- `troubleshooting.md` — recovery from common issues
- `../00_overview/what_is_continuum.md` — conceptual overview
