(async function () {
    const loginScreen = document.getElementById("login-screen");
    const loginSection = document.getElementById("login-section");
    const mobileNotice = document.getElementById("mobile-notice");
    const canvas = document.getElementById("app_canvas");
    const spinner = document.getElementById("loading_text");

    const shortestSide = Math.min(window.innerWidth, window.innerHeight);
    const isScreenTooSmall = shortestSide < 600;
    const isPhoneUserAgent = /iPhone|iPod|Android.*Mobile/i.test(navigator.userAgent);
    const isMobile = isScreenTooSmall || isPhoneUserAgent;
    if (isMobile) {
        spinner.style.display = "none";
        loginScreen.style.display = "flex";
        loginSection.style.display = "none";
        mobileNotice.style.display = "block";
        return;
    }

    try {
        const res = await fetch("/api/me", {credentials: "same-origin"});
        spinner.style.display = "none";

        if (res.ok) {
            canvas.style.display = "block";
            canvas.addEventListener("contextmenu", (e) => e.preventDefault());
        } else {
            loginScreen.style.display = "flex";
        }
    } catch (err) {
        console.error("Auth check failed:", err);
        spinner.style.display = "none";
        loginScreen.style.display = "flex";
    }
})();