const api = () => window.__TAURI__;
const invoke = (cmd, args) => api().core.invoke(cmd, args);

const LADDER = [
  ["coldread", "Cold read / voice"],
  ["fiction-dev-editor", "Developmental"],
  ["fiction-reviewchapter", "Style-guide review"],
  ["fiction-aiism-editor", "AI tells"],
  ["fiction-prose-editor", "Prose mechanics"],
  ["fiction-line-editor", "Line edit"],
  ["levelup", "Filter words"],
  ["fragment-hunter", "Fragments"],
  ["nominalization-hunt", "Nominalizations"],
  ["kill-chapter", "Kill-chapter pack"],
  ["burstiness-check", "Burstiness (report only)"],
];

const state = {
  project: null,
  status: null,
  jobId: null,
  file: null,
  models: [],
  skills: null,
  paletteRows: [],
  paletteIndex: 0,
  clearKey: false,
  enabledOverlays: [],
};

function $(id) {
  return document.getElementById(id);
}

function toast(msg) {
  const el = $("toast");
  el.textContent = msg;
  el.classList.remove("hidden");
  clearTimeout(toast._t);
  toast._t = setTimeout(() => el.classList.add("hidden"), 4000);
}

/** Tauri rejects with the serialized error string, not an Error. */
function errText(err) {
  return err && err.message ? err.message : String(err);
}

function esc(text) {
  return String(text).replace(
    /[&<>"]/g,
    (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;" })[c]
  );
}

function showView(name) {
  document.querySelectorAll("main.view").forEach((el) => el.classList.add("hidden"));
  $(`view-${name}`).classList.remove("hidden");
  document.querySelectorAll("nav.bottom button").forEach((btn) => {
    btn.classList.toggle("active", btn.dataset.view === name);
  });
}

async function loadSettings() {
  const s = await invoke("get_settings");
  applySettings(s);
  $("set-budget").value = s.token_budget || 48000;
  $("set-skills").value = s.skills_dir || "";
  state.enabledOverlays = s.enabled_overlays || [];
  document.querySelectorAll("[data-overlay]").forEach((el) => {
    el.checked = state.enabledOverlays.includes(el.dataset.overlay);
  });
  if (s.last_project && !state.project) {
    await openProject(s.last_project);
  }
  await refreshAuth();
}

/**
 * Take a settings reply from the host: the profile list, which one is active,
 * and which have a key. Draws the active profile into the form.
 */
function applySettings(s) {
  state.settings = s;
  state.profiles = (s.profiles || []).map((profile) => ({ ...profile }));
  state.activeProfile = s.active_profile;
  state.keyedProfiles = new Set(s.keyed_profiles || []);
  state.secretBackendLabel = s.secret_backend_label || "the secret store";
  renderProfiles();
  showProfile(state.activeProfile);
}

function currentProfile() {
  return state.profiles.find((p) => p.id === state.activeProfile) || state.profiles[0];
}

function renderProfiles() {
  $("set-profile").innerHTML = state.profiles
    .map((p) => `<option value="${escapeAttr(p.id)}">${escapeHtml(p.name || p.id)}</option>`)
    .join("");
  $("set-profile").value = state.activeProfile;
  $("btn-profile-delete").disabled = state.profiles.length < 2;
}

/** Draw one profile into the form. A half-typed key never follows a switch. */
function showProfile(id) {
  state.activeProfile = id;
  const p = currentProfile();
  $("set-profile").value = p.id;
  $("set-profile-name").value = p.name || "";
  $("set-provider").value = p.provider;
  $("set-base").value = p.base_url;
  $("set-style").value = p.api_style;
  $("set-model").value = p.model;
  $("set-cheap").value = p.cheap_model || "";
  $("set-routes").value = Object.entries(p.skill_models || {})
    .map(([skill, model]) => `${skill}=${model}`)
    .join("\n");
  // The key is write-only: the host never sends it back, so the field starts
  // empty and only ever carries a new value up.
  $("set-key").value = "";
  state.clearKey = false;
  renderKeyState();
  // A loaded model list belongs to one provider; keep it across a save, not
  // across a switch.
  if (state.shownProfile !== p.id) {
    state.shownProfile = p.id;
    state.models = [];
    $("models-state").textContent = "Paste a URL and key above, then load what the provider offers.";
    fillModelPickers([], false);
  }
}

/** Copy the form back into the profile it shows. */
function stashProfile() {
  const p = currentProfile();
  if (!p) return;
  p.name = $("set-profile-name").value.trim() || p.id;
  p.provider = $("set-provider").value;
  p.base_url = $("set-base").value.trim();
  p.api_style = $("set-style").value;
  p.model = $("set-model").value.trim();
  p.cheap_model = $("set-cheap").value.trim() || null;
  p.skill_models = parseRoutes($("set-routes").value);
}

function newProfile() {
  stashProfile();
  const id = `p-${Date.now().toString(36)}`;
  state.profiles.push({
    id,
    name: "New profile",
    provider: "openai-compat",
    base_url: "",
    api_style: "chat_completions",
    model: "",
    cheap_model: null,
    skill_models: {},
  });
  renderProfiles();
  showProfile(id);
  $("set-profile-name").select();
  toast("New profile: fill it in, then Save settings");
}

function deleteProfile() {
  if (state.profiles.length < 2) return;
  const p = currentProfile();
  if (!confirm(`Delete the profile "${p.name}" and its stored key?`)) return;
  state.profiles = state.profiles.filter((other) => other.id !== p.id);
  renderProfiles();
  showProfile(state.profiles[0].id);
  toast("Profile removed: Save settings to make it final");
}

/**
 * Where the active profile's key lives, in words. The key itself never
 * reaches the page.
 */
function renderKeyState() {
  const where = state.secretBackendLabel || "the secret store";
  if (state.clearKey) {
    $("key-state").textContent = "This profile's stored key will be forgotten when you save.";
    return;
  }
  $("key-state").textContent = state.keyedProfiles?.has(state.activeProfile)
    ? `A key is stored for this profile in ${where}. Type a new one to replace it.`
    : `No key stored for this profile yet. It will go to ${where}.`;
}

/** The settings form as the host expects it. Unsaved edits included. */
function collectSettings() {
  stashProfile();
  return {
    ...state.settings,
    last_project: state.project,
    profiles: state.profiles,
    active_profile: state.activeProfile,
    api_key: $("set-key").value.trim() || null,
    clear_api_key: state.clearKey,
    token_budget: Number($("set-budget").value) || 48000,
    enabled_overlays: [...document.querySelectorAll("[data-overlay]:checked")].map(
      (el) => el.dataset.overlay
    ),
    skills_dir: $("set-skills").value.trim() || null,
  };
}

async function saveSettings() {
  const next = collectSettings();
  state.enabledOverlays = next.enabled_overlays;
  const saved = await invoke("save_settings", { settings: next });
  // The key is now in the secret store, not in the form.
  applySettings(saved);
  if (state.project) await renderLadder();
  toast(`Settings saved — using "${currentProfile().name}"`);
}

function escapeHtml(text) {
  return String(text).replace(/[&<>"']/g, (c) => `&#${c.charCodeAt(0)};`);
}
const escapeAttr = escapeHtml;

/**
 * Home leads with the status board and the jobs list; the folder controls only
 * come first while there is no book to show.
 */
function placeBookCard() {
  const home = $("view-home");
  const card = $("card-book");
  if (state.project) home.append(card);
  else home.prepend(card);
}

async function openProject(path) {
  let found;
  try {
    found = await invoke("discover_projects", { start: path });
  } catch (err) {
    // discover_projects errors when the folder holds more than one book.
    toast(errText(err));
    return;
  }
  if (!found.length) {
    toast("No Wiki/ folder there");
    return;
  }
  if (found.length > 1) {
    toast("Multiple books found; pick the inner folder");
    return;
  }
  state.project = found[0].path;
  $("toast").classList.add("hidden"); // an error from an earlier attempt is stale now
  $("subtitle").textContent = "Status is read from disk every time";
  $("project-path").textContent = state.project;
  $("open-path").value = state.project;
  state.file = null;
  placeBookCard();
  await showTitle(found[0]);
  const settings = await invoke("get_settings");
  settings.last_project = state.project;
  await invoke("save_settings", { settings });
  await refreshAll();
}

/** The book's working title, or the folder name until it has one. */
async function showTitle(book) {
  const title = book.title || book.path.split(/[\\/]/).pop();
  if ($("title").textContent === title) return;
  $("title").textContent = title;
  // Desktop: the taskbar should say which book is open. Best effort — the
  // capability may not be granted.
  try {
    await api().window.getCurrentWindow().setTitle(`${title} — Open Storycraft`);
  } catch {
    /* title bar keeps the static name */
  }
}

async function refreshAll() {
  if (!state.project) return;
  // An import or a genre run can give the book its title; read it again.
  try {
    const [book] = await invoke("discover_projects", { start: state.project });
    if (book) await showTitle(book);
  } catch {
    /* keep the title already shown */
  }
  const status = await invoke("get_status", { project: state.project, chapter: null, mode: "resume" });
  state.status = status;
  renderSlots(status);
  renderBible();
  renderChapterLabels();
  const next = $("btn-next");
  if (status.next_skill) {
    next.disabled = false;
    next.textContent = `Next: ${status.next_skill}`;
  } else {
    next.disabled = true;
    next.textContent = "Spine complete";
  }
  await Promise.all([renderJobs(), renderFiles(), renderChapters(), renderLadder()]);
}

/**
 * The bible card: Write drafts one from the book, Import unpacks it into Wiki
 * files. Import needs a storybible.md on disk, so it stays off without one.
 */
function renderBible() {
  const bible = state.status?.storybible || null;
  $("btn-bible-import").disabled = !bible;
  $("btn-bible-convert").disabled = !bible;
  $("bible-state").textContent = bible
    ? `${bible} — Import writes its documents into this book folder. If your bible is in your own format, Convert it first (the original is kept as storybible.source.md).`
    : "No storybible.md here yet. Write drafts one from the Wiki, or put your own bible here as storybible.md and Convert it; Import unpacks a bible into Wiki files.";
}

/**
 * Say which chapter the board is talking about. Scenes, psych, chapters and
 * every editorial pass are chapter-scoped, and the board moves to the next
 * chapter as soon as one is drafted.
 */
function renderChapterLabels() {
  const chapter = state.status?.chapter;
  $("status-chapter").textContent = chapter ? `chapter ${chapter}` : "";
  $("edit-chapter").textContent = chapter ? `chapter ${chapter}` : "";
  $("btn-burstiness").textContent = chapter ? `Measure chapter ${chapter}` : "Measure this chapter";
}

/** The skill that fills each board slot, from the orchestrator's spine. */
const SLOT_SKILL = {
  genre: "fiction-genre",
  audience: "fiction-audience",
  theme: "fiction-theme",
  synopsis: "fiction-synopsis",
  style: "fiction-style",
  characters: "fiction-characters",
  world: "fiction-world",
  outline: "fiction-outline",
  scenes: "fiction-scenes",
  voice: "fiction-voiceprompt",
  psych: "fiction-psych",
  chapters: "fiction-writechapter",
};

function renderSlots(status) {
  if (!state.project) {
    $("slots").innerHTML =
      '<li class="muted">Open a book folder to see the board.</li>';
    return;
  }
  $("slots").innerHTML = status.slots
    .map((slot) => {
      const skill = SLOT_SKILL[slot.name];
      const title = skill
        ? `${slotHint(slot.name)} — click to run ${skill}`
        : slotHint(slot.name);
      return `<li data-skill="${skill || ""}" title="${esc(title)}"><span>${slot.name}</span><span class="${slot.state}">${slot.state}</span></li>`;
    })
    .join("");
  // The board is the map of what is missing, so let it fill itself.
  $("slots").onclick = (ev) => {
    const li = ev.target.closest("li[data-skill]");
    if (li?.dataset.skill) runSkill(li.dataset.skill, [], state.status?.chapter || 1);
  };
}

/** What each board slot actually is, on hover. */
function slotHint(name) {
  const hints = {
    genre: "Wiki/Style/genre.md — genre, subgenre, tropes",
    audience: "Wiki/Style/audience.md — who the book is for",
    theme: "Wiki/Story/theme.md — the question the book argues with",
    synopsis: "Wiki/Story/synopsis.md — the story contract",
    style: "Wiki/Style/style_guide.md — POV, tense, prose rules",
    characters: "Wiki/Characters/ — one sheet per character",
    world: "Wiki/Locations, Organizations, Systems, Events — optional",
    outline: "Wiki/Outline/outline.md — chapter by chapter",
    scenes: "Scene beats for the current chapter",
    voice: "Wiki/Style/voice_prompt.md — the POV voice firewall",
    psych: "Interior pass for the current chapter",
    chapters: "Drafted prose for the current chapter",
  };
  return hints[name] || name;
}

async function renderJobs() {
  const jobs = await invoke("list_jobs", { project: state.project });
  $("jobs").innerHTML =
    jobs
      .slice()
      .reverse()
      .slice(0, 12)
      .map(
        (job) =>
          `<li data-id="${esc(job.id)}"><span>${esc(job.skill)}</span><span class="pill ${esc(job.status)}">${esc(job.status)}</span></li>`
      )
      .join("") || "<li class='muted'>No jobs yet</li>";
  $("jobs").onclick = (ev) => {
    const li = ev.target.closest("li[data-id]");
    if (li) openJob(li.dataset.id);
  };
}

async function renderFiles() {
  const files = await invoke("list_files", { project: state.project });
  const entries = files.filter((f) => f.kind === "file");
  $("files").innerHTML =
    entries
      .map((f) => `<li data-rel="${esc(f.rel)}">${esc(f.rel)}</li>`)
      .join("") || "<li class='muted'>No files</li>";
  $("files").onclick = (ev) => {
    const li = ev.target.closest("li[data-rel]");
    if (li) showFile(li.dataset.rel);
  };
  // Always show something: re-open the current file, else the first one.
  const wanted = entries.some((f) => f.rel === state.file) ? state.file : entries[0]?.rel;
  if (wanted) await showFile(wanted);
}

async function showFile(rel) {
  state.file = rel;
  $("files")
    .querySelectorAll("li[data-rel]")
    .forEach((li) => li.classList.toggle("selected", li.dataset.rel === rel));
  try {
    $("file-preview").textContent = await invoke("read_project_file", {
      project: state.project,
      rel,
    });
  } catch (err) {
    $("file-preview").textContent = errText(err);
  }
}

async function renderChapters() {
  const files = await invoke("list_files", { project: state.project });
  const outline = files.find((f) => f.rel.endsWith("Outline/outline.md"));
  let chapters = [];
  if (outline) {
    const text = await invoke("read_project_file", { project: state.project, rel: outline.rel });
    for (const line of text.split("\n")) {
      const m = line.match(/^## Chapter\s+(\d+)/i);
      if (m) chapters.push(Number.parseInt(m[1], 10));
    }
  }
  if (!chapters.length) chapters = [state.status?.chapter || 1];
  $("chapters").innerHTML = chapters
    .map((n) => `<li data-ch="${n}">Chapter ${n}<button type="button" data-draft="${n}">Draft</button></li>`)
    .join("");
  $("chapters").onclick = (ev) => {
    const btn = ev.target.closest("[data-draft]");
    if (btn) runSkill("fiction-writechapter", [], Number(btn.dataset.draft));
  };
}

/**
 * Fill a picker from a fetched model list. The text input stays the source of
 * truth: choosing an option copies into it, and a hand-typed id the provider
 * does not list is kept, marked as such.
 */
function fillModelPicker(pickId, inputId, models, loaded) {
  const pick = $(pickId);
  const current = $(inputId).value.trim();
  const options = [];
  if (current && !models.includes(current)) {
    const note = loaded ? "not in the provider list" : "saved";
    options.push(
      `<option value="${esc(current)}" selected>${esc(current)} — ${note}</option>`
    );
  }
  for (const id of models) {
    options.push(
      `<option value="${esc(id)}"${id === current ? " selected" : ""}>${esc(id)}</option>`
    );
  }
  pick.innerHTML =
    options.length > 0 ? options.join("") : '<option value="">No models returned</option>';
}

function fillModelPickers(models, loaded) {
  fillModelPicker("set-model-pick", "set-model", models, loaded);
  fillModelPicker("set-cheap-pick", "set-cheap", models, loaded);
}

/** Ask the provider what it serves, using the URL and key in the form. */
async function loadModels() {
  const settings = await collectSettings();
  const button = $("btn-models");
  button.disabled = true;
  const baseUrl = currentProfile().base_url;
  $("models-state").textContent = `Loading models from ${baseUrl}…`;
  try {
    const models = await invoke("list_models", { settings });
    state.models = models;
    fillModelPickers(models, true);
    $("models-state").textContent = `${models.length} models from ${baseUrl}`;
  } catch (err) {
    // Keep the last good list: a failed refresh must not throw away options
    // that already worked.
    fillModelPickers(state.models, state.models.length > 0);
    $("models-state").textContent = `Could not list models: ${errText(err)}`;
  } finally {
    button.disabled = false;
  }
}

function parseRoutes(text) {
  const out = {};
  for (const line of text.split("\n")) {
    const trimmed = line.trim();
    if (!trimmed || trimmed.startsWith("#")) continue;
    const i = trimmed.indexOf("=");
    if (i <= 0) continue;
    const skill = trimmed.slice(0, i).trim();
    const model = trimmed.slice(i + 1).trim();
    if (skill && model) out[skill] = model;
  }
  return out;
}

async function renderLadder() {
  let extra = [];
  try {
    await loadSkills();
    const enabled = new Set(state.enabledOverlays || []);
    extra = (state.skills || [])
      .filter((skill) => skill.overlay && enabled.has(skill.name))
      .map((skill) => [skill.name, `${skill.name} (overlay)`]);
  } catch {
    extra = [];
  }
  const rows = LADDER.concat(extra);
  $("ladder").innerHTML = rows
    .map(
      ([skill, label]) =>
        `<li data-skill="${skill}">${label}<button type="button" data-run="${skill}">Run</button></li>`
    )
    .join("");
  $("ladder").onclick = (ev) => {
    const btn = ev.target.closest("[data-run]");
    if (btn) runSkill(btn.dataset.run, [], state.status?.chapter || 1);
  };
}

async function runSkill(skill, answers, chapter) {
  if (!state.project) {
    toast("Open a book first");
    return;
  }
  $("job-card").classList.remove("hidden");
  $("job-title").textContent = skill;
  setJobState("running");
  $("btn-apply-job").classList.add("hidden");
  $("job-preview").classList.remove("hidden");
  $("job-preview").textContent = "";
  $("job-diff").classList.add("hidden");
  try {
    const job = await invoke("run_skill", {
      args: { project: state.project, skill, answers, chapter, commit: false },
    });
    state.jobId = job.id;
    await openJob(job.id);
  } catch (err) {
    // The card is already on screen and carries the failure; a toast on top of
    // it would just say the same thing twice.
    setJobState("failed");
    $("job-preview").textContent = errText(err);
  }
  await renderJobs();
}

async function openJob(id) {
  const detail = await invoke("get_job", { project: state.project, id });
  state.jobId = id;
  $("job-card").classList.remove("hidden");
  $("job-title").textContent = detail.job.skill;
  setJobState(detail.job.status);
  $("job-preview").classList.remove("hidden");
  $("job-diff").classList.add("hidden");
  $("job-preview").textContent = detail.preview || detail.job.error || "";
  // A saved findings report can be applied to the chapter as its own run.
  const appliable = REPORT_SKILLS.has(detail.job.skill) && detail.job.status === "saved";
  $("btn-apply-job").classList.toggle("hidden", !appliable);
  $("btn-apply-job").onclick = () =>
    runSkill(`${detail.job.skill}:apply`, [], detail.job.chapter);
}

/** Editorial passes that write a report beside the chapter (see chunk.rs). */
const REPORT_SKILLS = new Set(["fiction-line-editor", "fragment-hunter"]);

/**
 * Save / Diff / Reject only mean something for a preview waiting on a decision.
 * The host rejects them for any other status, so do not offer them.
 */
function setJobState(status) {
  const pill = $("job-state");
  pill.textContent = status;
  pill.className = `pill ${status}`;
  // Until the first word arrives, say how long it has been: reasoning models
  // can think for minutes before writing, and a silent card looks frozen.
  clearInterval(setJobState._timer);
  $("job-wait").classList.add("hidden");
  if (status === "running") {
    const started = Date.now();
    setJobState._timer = setInterval(() => {
      const waiting = $("job-preview").textContent.length === 0;
      $("job-wait").classList.toggle("hidden", !waiting);
      if (!waiting) return;
      const secs = Math.round((Date.now() - started) / 1000);
      const time = secs < 60 ? `${secs}s` : `${Math.floor(secs / 60)}m ${secs % 60}s`;
      $("job-wait").textContent = `Waiting for the first words… ${time}. Reasoning models think before they write; the run stops by itself if the provider goes silent for 2 minutes.`;
    }, 1000);
  }
  const confirmable = status === "needs_confirm";
  for (const id of ["btn-save-job", "btn-diff-job", "btn-reject-job"]) {
    $(id).classList.toggle("hidden", !confirmable);
  }
}

async function refreshAuth() {
  try {
    $("auth-status").textContent = await invoke("auth_status");
  } catch (err) {
    $("auth-status").textContent = String(err);
  }
}

/**
 * Command palette: every skill in the pack, filtered as you type, run on
 * Enter. Overlay skills stay hidden until they are enabled in Settings.
 */
function paletteSkills(query) {
  const enabled = new Set(state.enabledOverlays || []);
  const needle = query.trim().toLowerCase();
  return (state.skills || [])
    .filter((skill) => !skill.overlay || enabled.has(skill.name))
    .filter(
      (skill) =>
        !needle ||
        skill.name.toLowerCase().includes(needle) ||
        (skill.description || "").toLowerCase().includes(needle)
    )
    .slice(0, 60);
}

function renderPalette(query) {
  const rows = paletteSkills(query);
  state.paletteRows = rows;
  if (state.paletteIndex >= rows.length) state.paletteIndex = 0;
  $("palette-list").innerHTML =
    rows
      .map((skill, index) => {
        const tag = skill.local ? '<span class="tag">on device</span>' : "";
        return `<li data-index="${index}"${index === state.paletteIndex ? ' aria-selected="true"' : ""}>
          <span class="name">${esc(skill.name)} ${tag}</span>
          <span class="desc">${esc(skill.description || "")}</span>
        </li>`;
      })
      .join("") || '<li class="muted">No skill matches.</li>';
  const active = $("palette-list").querySelector('li[aria-selected="true"]');
  if (active) active.scrollIntoView({ block: "nearest" });
  $("palette-hint").textContent = `${rows.length} of ${(state.skills || []).length} skills · ↑↓ to choose · Enter to run · Esc to close`;
}

async function loadSkills() {
  if (state.skills?.length) return;
  try {
    state.skills = await invoke("list_skills");
  } catch {
    state.skills = [];
  }
}

async function openPalette() {
  if (!state.project) {
    toast("Open a book folder first");
    return;
  }
  await loadSkills();
  state.paletteIndex = 0;
  $("palette").classList.remove("hidden");
  $("palette-input").value = "";
  renderPalette("");
  $("palette-input").focus();
}

function closePalette() {
  $("palette").classList.add("hidden");
}

function runPaletteSelection() {
  const skill = state.paletteRows?.[state.paletteIndex];
  if (!skill) return;
  closePalette();
  runSkill(skill.name, [], state.status?.chapter || 1);
}

function bindPalette() {
  $("btn-palette").onclick = openPalette;
  const input = $("palette-input");
  input.addEventListener("input", () => {
    state.paletteIndex = 0;
    renderPalette(input.value);
  });
  input.addEventListener("keydown", (ev) => {
    if (ev.key === "ArrowDown" || ev.key === "ArrowUp") {
      ev.preventDefault();
      const rows = state.paletteRows || [];
      if (!rows.length) return;
      const step = ev.key === "ArrowDown" ? 1 : -1;
      state.paletteIndex = (state.paletteIndex + step + rows.length) % rows.length;
      renderPalette(input.value);
      return;
    }
    if (ev.key === "Enter") {
      ev.preventDefault();
      runPaletteSelection();
      return;
    }
    if (ev.key === "Escape") {
      ev.preventDefault();
      ev.stopPropagation();
      closePalette();
    }
  });
  $("palette-list").onclick = (ev) => {
    const li = ev.target.closest("li[data-index]");
    if (!li) return;
    state.paletteIndex = Number(li.dataset.index);
    runPaletteSelection();
  };
}

/** Ctrl+1..5 switch views, in the order they appear in the nav. */
const VIEW_KEYS = ["home", "book", "write", "edit", "settings"];

/**
 * Desktop keyboard: view switching, the board, the job card. Every binding
 * mirrors a button that is already on screen.
 */
function bindShortcuts() {
  document.addEventListener("keydown", (ev) => {
    const mod = ev.ctrlKey || ev.metaKey;
    const key = ev.key.toLowerCase();
    if (mod && VIEW_KEYS[Number(ev.key) - 1]) {
      ev.preventDefault();
      showView(VIEW_KEYS[Number(ev.key) - 1]);
      return;
    }
    if (mod && key === "k") {
      ev.preventDefault();
      openPalette();
      return;
    }
    if (mod && key === "r") {
      ev.preventDefault();
      refreshAll();
      return;
    }
    if (mod && key === "o") {
      ev.preventDefault();
      $("btn-open").click();
      return;
    }
    if (mod && ev.key === "Enter") {
      ev.preventDefault();
      $("btn-next").click();
      return;
    }
    if (mod && key === "s") {
      ev.preventDefault();
      if (!$("btn-save-job").classList.contains("hidden")) $("btn-save-job").click();
      return;
    }
    if (ev.key === "Escape") {
      if (!$("palette").classList.contains("hidden")) {
        closePalette();
        return;
      }
      if (!$("job-card").classList.contains("hidden")) $("job-card").classList.add("hidden");
    }
  });
}

async function main() {
  if (!api()) {
    toast("Tauri bridge missing — open this UI from the desktop app");
    return;
  }
  if (navigator.userAgent.includes("Windows")) {
    $("open-path").placeholder = "C:\\Users\\you\\Documents\\Books\\salt-ledger";
  }
  document.querySelectorAll("nav.bottom button").forEach((btn) => {
    btn.addEventListener("click", () => showView(btn.dataset.view));
  });
  placeBookCard();
  bindPalette();
  bindShortcuts();
  $("btn-open").onclick = async () => {
    // Hand the chooser the open book (or the last one) so it does not land on
    // Recents, which is useless for a book kept deep in a home directory.
    const folder = await invoke("pick_folder", { start: state.project });
    if (folder) await openProject(folder);
  };
  $("btn-open-path").onclick = () => {
    const typed = $("open-path").value.trim();
    if (!typed) {
      toast("Type a folder path first");
      return;
    }
    openProject(typed);
  };
  $("open-path").addEventListener("keydown", (ev) => {
    if (ev.key === "Enter") $("btn-open-path").click();
  });
  $("btn-refresh").onclick = refreshAll;
  $("btn-bible-import").onclick = () =>
    runSkill("storybible-import", [], state.status?.chapter || 1);
  $("btn-bible-write").onclick = () =>
    runSkill("fiction-storybible", [], state.status?.chapter || 1);
  $("btn-bible-convert").onclick = () =>
    runSkill("storybible-convert", [], state.status?.chapter || 1);
  $("btn-next").onclick = () => {
    if (state.status?.next_skill) runSkill(state.status.next_skill, [], state.status.chapter);
  };
  $("btn-spark").onclick = () => {
    if (!state.project) {
      toast("Open a book folder first (spark will not write Wiki files)");
      return;
    }
    runSkill("fiction-story-sparks", [], 1);
  };
  $("btn-save-settings").onclick = saveSettings;
  $("btn-models").onclick = loadModels;
  // Marked here, applied by Save, like every other field on this screen.
  $("btn-key-clear").onclick = () => {
    state.clearKey = true;
    $("set-key").value = "";
    renderKeyState();
  };
  $("set-profile").onchange = (ev) => {
    stashProfile();
    showProfile(ev.target.value);
  };
  $("set-profile-name").oninput = () => {
    stashProfile();
    renderProfiles();
  };
  $("btn-profile-new").onclick = newProfile;
  $("btn-profile-delete").onclick = deleteProfile;
  $("set-model-pick").onchange = (ev) => {
    if (ev.target.value) $("set-model").value = ev.target.value;
  };
  $("set-cheap-pick").onchange = (ev) => {
    if (ev.target.value) $("set-cheap").value = ev.target.value;
  };
  $("btn-diff-job").onclick = async () => {
    if (!state.jobId) return;
    let diff;
    try {
      diff = await invoke("job_diff", { project: state.project, id: state.jobId });
    } catch (err) {
      toast(errText(err));
      return;
    }
    $("job-preview").classList.add("hidden");
    $("job-diff").classList.remove("hidden");
    $("job-diff").textContent = diff || "no changes";
  };
  $("btn-burstiness").onclick = async () => {
    if (!state.project) {
      toast("Open a book first");
      return;
    }
    try {
      const report = await invoke("burstiness_report", {
        project: state.project,
        chapter: state.status?.chapter || 1,
      });
      $("burst-out").textContent =
        `sentences ${report.sentence_count}\n` +
        `paragraphs ${report.paragraph_count}\n` +
        `sentence variance ${report.sentence_length_variance_bucket}\n` +
        `dialogue ${report.dialogue_word_ratio}\n` +
        `interiority ${report.interiority_risk_level} (${report.interiority_risk_score})`;
    } catch (err) {
      toast(String(err));
    }
  };
  $("btn-export").onclick = async () => {
    if (!state.project) {
      toast("Open a book first");
      return;
    }
    const dest = await invoke("export_project", { project: state.project });
    toast(`exported ${dest}`);
  };
  $("btn-save-job").onclick = async () => {
    if (!state.jobId) return;
    try {
      const dest = await invoke("save_job", { project: state.project, id: state.jobId });
      toast(dest ? `saved ${dest}` : "accepted; no Wiki write");
      await refreshAll();
      await openJob(state.jobId);
    } catch (err) {
      toast(errText(err));
    }
  };
  $("btn-reject-job").onclick = async () => {
    if (!state.jobId) return;
    try {
      await invoke("reject_job", { project: state.project, id: state.jobId });
    } catch (err) {
      toast(errText(err));
      return;
    }
    toast("rejected");
    $("job-card").classList.add("hidden");
    await renderJobs();
  };
  $("btn-close-job").onclick = () => $("job-card").classList.add("hidden");
  $("btn-auth").onclick = async () => {
    const dto = await invoke("auth_login");
    $("auth-code").textContent = dto.user_code;
    toast("Authorize in the browser, then tap I've authorized");
  };
  $("btn-auth-poll").onclick = async () => {
    await invoke("auth_poll");
    toast("signed in");
    await refreshAuth();
  };
  $("btn-auth-out").onclick = async () => {
    await invoke("auth_logout");
    await refreshAuth();
  };
  await api().event.listen("job-delta", (event) => {
    if (event.payload?.chunk) $("job-preview").textContent += event.payload.chunk;
  });
  await loadSettings();
}

main().catch((err) => toast(String(err)));
