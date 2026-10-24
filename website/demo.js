const keys = {
    ArrowUp: "north",
    w: "north",
    k: "north",
    ArrowDown: "south",
    s: "south",
    j: "south",
    ArrowLeft: "west",
    a: "west",
    h: "west",
    ArrowRight: "east",
    d: "east",
    l: "east",
    e: "interact",
    f: "finish",
};

let mounted = false;

export function mountDemo() {
    if (mounted) return;
    mounted = true;
    const canvas = document.getElementById("game-map");
    const message = document.getElementById("demo-message");
    const entry = document.getElementById("demo-entry");
    const startButton = document.getElementById("start-demo");
    const restartButton = document.getElementById("restart-demo");
    const finishButton = document.getElementById("finish-demo");
    const movement = [...document.querySelectorAll("[data-action]")];
    let Demo;
    let game;
    let state;
    let loading = false;

    function show(next) {
        state = JSON.parse(next);
        document.getElementById("stat-depth").textContent =
            `${state.floor} / ${state.floors}`;
        document.getElementById("stat-charge").textContent =
            `${state.charge} / ${state.capacity}`;
        document.getElementById("stat-score").textContent = String(state.score);
        document.getElementById("stat-turns").textContent =
            `${state.turns} / ${state.max_turns}`;
        message.textContent = state.message;
        movement.forEach((button) => button.disabled = state.finished);
        finishButton.disabled = state.finished;
        paintMap(canvas, state);
        canvas.setAttribute(
            "aria-label",
            `Castle map, depth ${state.floor} of ${state.floors}. You are at column ${
                state.position.x + 1
            }, row ${
                state.position.y + 1
            }. Score ${state.score}, charge ${state.charge}. ${nearby(state)}`,
        );
    }

    function act(command) {
        if (!game || state?.finished) return;
        try {
            show(game.act(command));
        } catch (error) {
            message.textContent =
                "This descent stopped unexpectedly. Choose New descent to start again.";
            movement.forEach((button) => button.disabled = true);
            finishButton.disabled = true;
            state.finished = true;
            console.error("Demo action failed:", error);
        }
    }

    async function start() {
        if (loading) return;
        loading = true;
        startButton.disabled = restartButton.disabled = true;
        startButton.textContent = "> Lighting the lantern…";
        message.textContent = "Preparing the practice expedition…";
        try {
            if (!globalThis.WebAssembly) {
                throw new Error("WebAssembly is unavailable in this browser.");
            }
            if (!Demo) {
                const module = await import("./pkg/diggerchain_web.js");
                await module.default();
                Demo = module.Demo;
            }
            const words = crypto.getRandomValues(new Uint32Array(2));
            const seed = [...words].map((word) =>
                word.toString(16).padStart(8, "0")
            ).join("");
            const next = new Demo(seed);
            game?.free();
            game = next;
            entry.hidden = true;
            canvas.hidden = false;
            show(game.state());
            canvas.focus({ preventScroll: true });
            restartButton.disabled = false;
        } catch (error) {
            message.textContent =
                "The demo could not start. Try again, or use a current browser with WebAssembly enabled.";
            startButton.textContent = "> Try again";
            startButton.disabled = false;
            restartButton.disabled = !game;
            console.error("Demo initialization failed:", error);
        } finally {
            loading = false;
        }
    }

    startButton.addEventListener("click", start);
    restartButton.addEventListener("click", start);
    finishButton.addEventListener("click", () => act("finish"));
    for (const button of movement) {
        button.addEventListener("click", () => act(button.dataset.action));
    }
    canvas.addEventListener("keydown", (event) => {
        if (event.altKey || event.ctrlKey || event.metaKey) return;
        const command = keys[event.key] || keys[event.key.toLowerCase()];
        if (!command) return;
        event.preventDefault();
        if (!event.repeat) act(command);
    });
    canvas.addEventListener(
        "pointerdown",
        () => canvas.focus({ preventScroll: true }),
    );
    document.fonts.ready.then(() => {
        if (state) paintMap(canvas, state);
    });
    document.addEventListener("diggerchain:themechange", () => {
        if (state) paintMap(canvas, state);
    });
    new ResizeObserver(() => {
        if (state) paintMap(canvas, state);
    }).observe(canvas);
    window.addEventListener("resize", () => {
        if (state) paintMap(canvas, state);
    });
}

function nearby(state) {
    const descriptions = {
        "#": "wall",
        ".": "open floor",
        r: "soft vein",
        R: "hard vein",
        C: "crystal",
        F: "flask",
        "<": "up portal",
        ">": "down portal",
        " ": "unexplored",
    };
    return [["North", 0, -1], ["South", 0, 1], ["West", -1, 0], ["East", 1, 0]]
        .map(([name, dx, dy]) => {
            const x = state.position.x + dx;
            const y = state.position.y + dy;
            const cell = x < 0 || y < 0 || x >= state.width || y >= state.height
                ? "#"
                : state.cells[y * state.width + x];
            return `${name}: ${descriptions[cell] || "remembered passage"}.`;
        }).join(" ");
}

function paintMap(canvas, state) {
    const { width, height } = canvas.getBoundingClientRect();
    if (!width || !height) return;
    const cellSize = Math.floor(
        Math.min(width / state.width, height / state.height, 32),
    );
    if (cellSize < 1) return;
    const ratio = Math.min(window.devicePixelRatio || 1, 3);
    const pixelWidth = Math.round(width * ratio);
    const pixelHeight = Math.round(height * ratio);
    if (canvas.width !== pixelWidth || canvas.height !== pixelHeight) {
        canvas.width = pixelWidth;
        canvas.height = pixelHeight;
    }
    const context = canvas.getContext("2d");
    if (!context) throw new Error("A 2D canvas is required for the demo.");
    context.setTransform(ratio, 0, 0, ratio, 0, 0);
    const style = getComputedStyle(document.documentElement);
    const tone = (name) => style.getPropertyValue(name).trim();
    const colors = {
        "#": tone("--map-wall"),
        ".": tone("--map-walk"),
        r: tone("--map-rock"),
        R: tone("--map-hard"),
        "*": tone("--color-crystal"),
        F: tone("--color-flask"),
        "<": tone("--color-portal"),
        ">": tone("--color-portal"),
        "@": tone("--color-ink"),
        ":": tone("--map-remembered"),
    };
    const floors = [tone("--map-floor"), tone("--map-floor-alt")];
    context.fillStyle = tone("--map-ground");
    context.fillRect(0, 0, width, height);
    context.save();
    context.translate(
        Math.floor((width - state.width * cellSize) / 2),
        Math.floor((height - state.height * cellSize) / 2),
    );
    context.textAlign = "center";
    context.textBaseline = "middle";
    context.font = `${
        Math.max(8, Math.round(cellSize * 0.7))
    }px "Libre Baskerville", Georgia, serif`;
    for (let y = 0; y < state.height; y++) {
        for (let x = 0; x < state.width; x++) {
            const i = y * state.width + x;
            let glyph = state.cells[i];
            if (glyph === " ") continue;
            const visible = state.visible[i];
            const room = Math.min(
                5,
                y < 6 ? Math.floor(x / 6) : 5 - Math.floor(x / 6),
            );
            const px = x * cellSize;
            const py = y * cellSize;
            context.fillStyle = visible
                ? floors[(room + state.floor) % floors.length]
                : tone("--map-memory");
            context.fillRect(px, py, cellSize, cellSize);
            if (glyph === "C") glyph = "*";
            let color = colors[glyph];
            if (visible && glyph === "#") {
                if (y === 0 && [3, 9, 15].includes(x)) {
                    glyph = "|";
                    color = colors["*"];
                } else if (y === 12 && [3, 9, 15].includes(x)) {
                    glyph = "!";
                    color = colors["<"];
                } else if (x === 18 && [3, 9].includes(y)) {
                    glyph = "t";
                    color = colors.F;
                } else if (x === 0 && y === 3) {
                    glyph = "=";
                    color = colors.r;
                } else if (x === 0 && y === 9) {
                    glyph = "+";
                    color = colors["<"];
                }
            }
            if (visible && glyph === ".") {
                if ((x === 16 && y === 1) || (x === 1 && y === 11)) {
                    glyph = "%";
                    color = colors["<"];
                } else if (x === 4 && y === 11) {
                    glyph = '"';
                    color = tone("--color-accent");
                }
            }
            context.fillStyle = visible ? (color || colors["#"]) : colors[":"];
            context.fillText(glyph, px + cellSize / 2, py + cellSize / 2);
        }
    }
    const px = state.position.x * cellSize;
    const py = state.position.y * cellSize;
    const inset = Math.max(1, Math.round(cellSize * 0.06));
    context.fillStyle = tone("--map-player-bg");
    context.fillRect(
        px + inset,
        py + inset,
        cellSize - inset * 2,
        cellSize - inset * 2,
    );
    context.fillStyle = colors["@"];
    context.fillText("@", px + cellSize / 2, py + cellSize / 2);
    context.restore();
}
