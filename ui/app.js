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
  $("set-skills").value = s.skills_dir || "";
  if (s.last_project && !state.project) {
    await openProject(s.last_project);
  }
  await refreshAuth();
}

async function saveSettings() {
  const current = await invoke("get_settings");
  const next = {
    ...current,
    last_project: state.project,
    provider: $("set-provider").value,
    base_url: $("set-base").value.trim(),
    api_key: $("set-key").value || null,
    api_style: $("set-style").value,
    model: $("set-model").value.trim(),
    skills_dir: $("set-skills").value.trim() || null,
  };
  await invoke("save_settings", { settings: next });
  toast("Settings saved");
}

async function openProject(path) {
  const found = await invoke("discover_projects", { start: path });
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
  $("jobs").innerHTML = jobs
    .slice()
    .reverse()
    .slice(0, 12)
    .map((job) => `<li data-id="${job.id}"><span>${job.skill}</span><span class="muted">${job.status}</span></li>`)
    .join("") || "<li class='muted'>No jobs yet</li>";
  $("jobs").onclick = (ev) => {
    const li = ev.target.closest("li[data-id]");
    if (li) openJob(li.dataset.id);
  };
}

async function renderFiles() {
  const files = await invoke("list_files", { project: state.project });
  $("files").innerHTML = files
    .filter((f) => f.kind === "file")
    .map((f) => `<li data-rel="${f.rel}">${f.rel}</li>`)
    .join("");
  $("files").onclick = async (ev) => {
    const li = ev.target.closest("li[data-rel]");
    if (!li) return;
    const text = await invoke("read_project_file", { project: state.project, rel: li.dataset.rel });
    $("file-preview").textContent = text;
  };
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

function renderLadder() {
  $("ladder").innerHTML = LADDER.map(
    ([skill, label]) => `<li data-skill="${skill}">${label}<button type="button" data-run="${skill}">Run</button></li>`
  ).join("");
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
  toast(`Running ${skill}…`);
  $("job-card").classList.remove("hidden");
  $("job-title").textContent = skill;
  $("job-state").textContent = "running";
  $("job-preview").textContent = "";
  try {
    const job = await invoke("run_skill", {
      args: { project: state.project, skill, answers, chapter, commit: false },
    });
    state.jobId = job.id;
    await openJob(job.id);
    toast(`${skill} ${job.status}`);
  } catch (err) {
    $("job-state").textContent = "failed";
    $("job-preview").textContent = String(err);
    toast(String(err));
  }
  await renderJobs();
}

async function openJob(id) {
  const detail = await invoke("get_job", { project: state.project, id });
  state.jobId = id;
  $("job-card").classList.remove("hidden");
  $("job-title").textContent = detail.job.skill;
  $("job-state").textContent = detail.job.status;
  $("job-preview").classList.remove("hidden");
  $("job-diff").classList.add("hidden");
  $("job-preview").textContent = detail.preview || detail.job.error || "";
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
  $("btn-open").onclick = async () => {
    const folder = await invoke("pick_folder");
    if (folder) await openProject(folder);
  };
  $("btn-refresh").onclick = refreshAll;
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
  $("btn-diff-job").onclick = async () => {
    if (!state.jobId) return;
    const diff = await invoke("job_diff", { project: state.project, id: state.jobId });
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
    const dest = await invoke("save_job", { project: state.project, id: state.jobId });
    toast(dest ? `saved ${dest}` : "accepted; no Wiki write");
    await refreshAll();
    await openJob(state.jobId);
  };
  $("btn-reject-job").onclick = async () => {
    if (!state.jobId) return;
    await invoke("reject_job", { project: state.project, id: state.jobId });
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
