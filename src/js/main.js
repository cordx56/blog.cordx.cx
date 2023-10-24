document.addEventListener("DOMContentLoaded", () => {
    if (window.matchMedia("(prefers-color-scheme: dark)").matches) {
        document.body.classList.add("dark");
    }
});

function toggleDark() {
    document.body.classList.toggle("dark");
}
