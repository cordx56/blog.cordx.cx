document.addEventListener("DOMContentLoaded", () => {
    const re = new RegExp("theme=(\\d+)");
    const matched = document.cookie.match(re);
    if (matched) {
        const theme = parseInt(matched[1]);
        if (theme === 1) {
            document.body.classList.add("dark");
        }
    } else if (window.matchMedia("(prefers-color-scheme: dark)").matches) {
        document.body.classList.add("dark");
    }
});

function toggleDark() {
    document.body.classList.toggle("dark");
    const current = document.body.classList.contains("dark") ? 1 : 0;
    document.cookie = `theme=${current};path=/`
}
