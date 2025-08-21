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
function onReady() {
    console.log("loaded!!!!");
}
