const pages = [...document.querySelectorAll("[data-page]")];
const mainLinks = [...document.querySelectorAll(".site-footer a")];
const commands = [...document.querySelectorAll(".boot-menu .command")];
const topicLinks = [...document.querySelectorAll(".guide-index a")];
const topics = topicLinks.map((link) => link.hash.split("/")[1]);
const chapters = [...document.querySelectorAll(".guide-copy > section")];
const topicSelect = document.getElementById("doc-topic");
const article = document.getElementById("guide-article");
const previousTopic = document.getElementById("doc-prev");
const nextTopic = document.getElementById("doc-next");
const themeButtons = [...document.querySelectorAll("[data-theme-choice]")];
const routes = new Set(pages.map((page) => page.id));
const readers = new Map();
let currentPage = "";
let currentTopic = "start";
let demoModule;
let selected = 0;

function focusContent() {
    const heading = currentPage === "docs"
        ? document.querySelector(`#${currentTopic} h2`)
        : readers.has(currentPage)
        ? readers.get(currentPage).pages[readers.get(currentPage).index]
            .querySelector("h2")
        : document.getElementById(currentPage).querySelector("h1");
    heading.setAttribute("tabindex", "-1");
    heading.focus({ preventScroll: true });
}

document.querySelector(".skip-link").addEventListener("click", (event) => {
    event.preventDefault();
    focusContent();
});

function selectCommand(index) {
    selected = (index + commands.length) % commands.length;
    commands.forEach((command, i) =>
        command.classList.toggle("selected", i === selected)
    );
}

commands.forEach((command, i) => {
    command.addEventListener("pointerenter", () => selectCommand(i));
    command.addEventListener("focus", () => selectCommand(i));
});

document.addEventListener("keydown", (event) => {
    if (
        currentPage !== "home" || event.altKey || event.ctrlKey || event.metaKey
    ) return;
    if (event.target.closest(".site-toolbar, .site-footer, .token-strip")) {
        return;
    }
    const change = { ArrowDown: 1, ArrowUp: -1 }[event.key];
    if (change) {
        event.preventDefault();
        selectCommand(selected + change);
        commands[selected].focus();
    } else if (event.key === "Enter" && !event.target.closest("a, button")) {
        event.preventDefault();
        commands[selected].click();
    }
});

function applyTheme(theme, save = false) {
    document.documentElement.dataset.theme = theme;
    themeButtons.forEach((button) =>
        button.setAttribute(
            "aria-pressed",
            String(button.dataset.themeChoice === theme),
        )
    );
    document.querySelector('meta[name="theme-color"]').content =
        getComputedStyle(document.documentElement).getPropertyValue(
            "--color-ground",
        ).trim();
    if (save) {
        try {
            localStorage.setItem("diggerchain-theme", theme);
        } catch { /* Theme switching still works without storage. */ }
    }
    document.dispatchEvent(new Event("diggerchain:themechange"));
}

themeButtons.forEach((button) =>
    button.addEventListener(
        "click",
        () => applyTheme(button.dataset.themeChoice, true),
    )
);
applyTheme(document.documentElement.dataset.theme || "dark");

topicSelect.replaceChildren(
    ...topicLinks.map((link, i) =>
        new Option(link.textContent.trim(), topics[i])
    ),
);
topicSelect.addEventListener("change", () => {
    location.hash = `docs/${topicSelect.value}`;
});
previousTopic.addEventListener("click", () => {
    location.hash = `docs/${topics[topics.indexOf(currentTopic) - 1]}`;
});
nextTopic.addEventListener("click", () => {
    location.hash = `docs/${topics[topics.indexOf(currentTopic) + 1]}`;
});

function selectTopic(requested) {
    const topic = topics.includes(requested) ? requested : currentTopic;
    const changed = currentTopic !== topic;
    currentTopic = topic;
    for (const chapter of chapters) chapter.hidden = chapter.id !== topic;
    topicLinks.forEach((link, index) => {
        if (topics[index] === topic) {
            link.setAttribute("aria-current", "location");
        } else link.removeAttribute("aria-current");
    });
    topicSelect.value = topic;
    const index = topics.indexOf(topic);
    previousTopic.disabled = index === 0;
    nextTopic.disabled = index === topics.length - 1;
    document.getElementById("doc-counter").textContent = `${
        index + 1
    } / ${topics.length}`;
    article.setAttribute("aria-labelledby", `${topic}-heading`);
    if (changed) article.scrollTop = 0;
    return changed;
}

for (const name of ["lore", "roadmap"]) {
    const root = document.getElementById(name);
    const reader = {
        pages: [...root.querySelectorAll("[data-reader-page]")],
        previous: root.querySelector("[data-reader-prev]"),
        next: root.querySelector("[data-reader-next]"),
        counter: root.querySelector("[data-reader-counter]"),
        links: [...root.querySelectorAll(".phase-nav a")],
        index: 0,
    };
    readers.set(name, reader);
    for (
        const [button, direction] of [[reader.previous, -1], [reader.next, 1]]
    ) {
        button.addEventListener("click", () => {
            const next = reader.pages[reader.index + direction];
            if (next) location.hash = `${name}/${next.dataset.slug}`;
        });
    }
    selectReaderPage(name);
}

function selectReaderPage(name, slug) {
    const reader = readers.get(name);
    if (!reader) return false;
    const requested = reader.pages.findIndex((page) =>
        page.dataset.slug === slug
    );
    const index = requested < 0 ? reader.index : requested;
    const changed = reader.index !== index;
    reader.index = index;
    reader.pages.forEach((page, i) => page.hidden = i !== index);
    reader.previous.disabled = index === 0;
    reader.next.disabled = index === reader.pages.length - 1;
    reader.counter.textContent = `${index + 1} / ${reader.pages.length}`;
    reader.links.forEach((link) => {
        if (link.hash === `#${name}/${reader.pages[index].dataset.slug}`) {
            link.setAttribute("aria-current", "step");
        } else link.removeAttribute("aria-current");
    });
    return changed;
}

for (const button of document.querySelectorAll("[data-copy-ca]")) {
    button.addEventListener("click", async () => {
        const address = button.parentElement.querySelector(
            "[data-token-address]",
        ).textContent.trim();
        if (!/^0x[0-9a-fA-F]{40}$/.test(address)) return;
        const original = button.textContent;
        try {
            await navigator.clipboard.writeText(address);
            button.textContent = "Copied";
        } catch {
            const range = document.createRange();
            range.selectNodeContents(
                button.parentElement.querySelector("[data-token-address]"),
            );
            const selection = window.getSelection();
            selection.removeAllRanges();
            selection.addRange(range);
            button.textContent = "Selected";
        }
        setTimeout(() => button.textContent = original, 1500);
    });
}

async function route(initial = false) {
    const [requested = "home", topic] = location.hash.slice(1).split("/");
    const name = routes.has(requested) ? requested : "home";
    const changed = currentPage !== name;
    currentPage = name;
    pages.forEach((page) => page.hidden = page.id !== name);
    mainLinks.forEach((link) => {
        if (link.hash === `#${name}`) link.setAttribute("aria-current", "page");
        else link.removeAttribute("aria-current");
    });
    const contentChanged = name === "docs"
        ? selectTopic(topic)
        : selectReaderPage(name, topic);
    const reader = readers.get(name);
    const slug = name === "docs"
        ? currentTopic
        : reader?.pages[reader.index].dataset.slug;
    if (slug && location.hash !== `#${name}/${slug}`) {
        history.replaceState(null, "", `#${name}/${slug}`);
    }
    if (!initial && (changed || contentChanged)) focusContent();
    if (name === "demo" && !demoModule) {
        try {
            demoModule = import("./demo.js");
            const demo = await demoModule;
            demo.mountDemo();
        } catch (error) {
            demoModule = undefined;
            document.getElementById("demo-message").textContent =
                "The demo could not load. Reload to try again, or download the terminal game.";
            console.error("Demo module could not load:", error);
        }
    }
}

window.addEventListener("hashchange", () => route());
selectTopic("start");
route(true);
