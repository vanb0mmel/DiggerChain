// Apply the saved palette before styles paint, including when storage is unavailable.
try {
    const theme = localStorage.getItem("diggerchain-theme");
    if (["dark", "nord", "solar"].includes(theme)) {
        document.documentElement.dataset.theme = theme;
    }
} catch {
    // The default palette remains usable in private or restricted browsing.
}
