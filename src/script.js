if (document.readyState === "complete") {
    onReady();
} else {
    window.addEventListener(
        "load",
        () => {
            onReady();
        },
        false,
        true,
    );
}
function waitForElm(selector) {
    return new Promise((resolve) => {
        if (document.querySelector(selector)) {
            return resolve(document.querySelector(selector));
        }

        const observer = new MutationObserver((mutations) => {
            if (document.querySelector(selector)) {
                observer.disconnect();
                resolve(document.querySelector(selector));
            }
        });

        // If you get "parameter 1 is not of type 'Node'" error, see https://stackoverflow.com/a/77855838/492336
        observer.observe(document.body, {
            childList: true,
            subtree: true,
        });
    });
}
async function onReady() {
    document
        .getElementsByClassName("mhy-hoyolab-account-block")[0]
        .dispatchEvent(new Event("click"));
    var element = await waitForElm("");
}
