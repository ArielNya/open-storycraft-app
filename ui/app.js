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
  $("set-provider").value = s.provider;
  $("set-base").value = s.base_url;
  $("set-key").value = s.api_key || "";
  $("set-style").value = s.api_style;
  $("set-model").value = s.model;
  $("set-cheap").value = s.cheap_model || "";
  $("set-budget").value = s.token_budget || 48000;
  $("set-routes").value = Object.entries(s.skill_models || {})
    .map(([skill, model]) => `${skill}=${model}`)
    .join("\n");
  $("set-skills").value = s.skills_dir || "";
  $("models-state").textContent = "Paste a URL and key above, then load what the provider offers.";
  fillModelPickers([], false);
  state.enabledOverlays = s.enabled_overlays || [];
  document.querySelectorAll("[data-overlay]").forEach((el) => {
    el.checked = state.enabledOverlays.includes(el.dataset.overlay);
  });
  if (s.last_project && !state.project) {
    await openProject(s.last_project);
  }
  await refreshAuth();
}

/** The settings form as the host expects it. Unsaved edits included. */
async function collectSettings() {
  const current = await invoke("get_settings");
  return {
    ...current,
    last_project: state.project,
    provider: $("set-provider").value,
    base_url: $("set-base").value.trim(),
    api_key: $("set-key").value || null,
    api_style: $("set-style").value,
    model: $("set-model").value.trim(),
    cheap_model: $("set-cheap").value.trim() || null,
    token_budget: Number($("set-budget").value) || 48000,
    skill_models: parseRoutes($("set-routes").value),
    enabled_overlays: [...document.querySelectorAll("[data-overlay]:checked")].map(
      (el) => el.dataset.overlay
    ),
    skills_dir: $("set-skills").value.trim() || null,
  };
}

async function saveSettings() {
  const next = await collectSettings();
  state.enabledOverlays = next.enabled_overlays;
  await invoke("save_settings", { settings: next });
  if (state.project) await renderLadder();
  toast("Settings saved");
}

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
  $("title").textContent = found[0].title || found[0].path.split("/").pop();
  $("subtitle").textContent = "Status is read from disk every time";
  $("project-path").textContent = state.project;
  $("open-path").value = state.project;
  state.file = null;
  placeBookCard();
  const settings = await invoke("get_settings");
  settings.last_project = state.project;
  await invoke("save_settings", { settings });
  await refreshAll();
}

async function refreshAll() {
  if (!state.project) return;
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
  $("bible-state").textContent = bible
    ? `${bible} — Import writes its documents into this book folder.`
    : "No storybible.md here yet. Write drafts one from the Wiki; Import unpacks a bible into Wiki files.";
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

function renderSlots(status) {
  $("slots").innerHTML = status.slots
    .map(
      (slot) =>
        `<li><span>${slot.name}</span><span class="${slot.state}">${slot.state}</span></li>`
    )
    .join("");
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
  $("models-state").textContent = `Loading models from ${settings.base_url}…`;
  try {
    const models = await invoke("list_models", { settings });
    state.models = models;
    fillModelPickers(models, true);
    $("models-state").textContent = `${models.length} models from ${settings.base_url}`;
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
    const skills = await invoke("list_skills");
    const enabled = new Set(state.enabledOverlays || []);
    extra = skills
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
}

/**
 * Save / Diff / Reject only mean something for a preview waiting on a decision.
 * The host rejects them for any other status, so do not offer them.
 */
function setJobState(status) {
  const pill = $("job-state");
  pill.textContent = status;
  pill.className = `pill ${status}`;
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

async function main() {
  if (!api()) {
    toast("Tauri bridge missing — open this UI from the desktop app");
    return;
  }
  document.querySelectorAll("nav.bottom button").forEach((btn) => {
    btn.addEventListener("click", () => showView(btn.dataset.view));
  });
  placeBookCard();
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
